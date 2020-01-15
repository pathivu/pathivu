/*
 * Copyright 2019 Balaji Jinnah and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use crate::config::config::Config;
use crate::iterator::merge_iterator::MergeIteartor;
use crate::json_parser::parser::get_value_from_json;
use crate::parser::parser;
use crate::partition::partition_iterator::PartitionIterator;
use crate::partition::segment_iterator::Entry;
use crate::server::server::PartitionHandler;
use crate::store::store::Store;
use crate::types::types::{IngesterFlushHintReq, IngesterRequest, STRUCTURED_DATA};
use crate::types::types::{QueryRequest, QueryResponse, ResLine};
use crate::util::util::{
    convert_static_node_to_f32, convert_static_node_to_string, convert_string_to_f32,
};
use failure::format_err;
use futures::channel::mpsc::Sender;
use futures::channel::oneshot;
use futures::executor::block_on;
use futures::sink::SinkExt;
use log::{debug, info, warn};
use serde_json::json;
use simd_json::value::borrowed::Value;
use simd_json::value::tape::StaticNode;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// DistinctRes gives the distinct value and count of the distinct value.
struct DistinctRes {
    distinct_count: HashMap<String, u64>,
    show_count: bool,
}

pub struct QueryExecutor<S: Store + Clone> {
    store: S,
    cfg: Config,
    ingester_transport: Sender<IngesterRequest>,
    partition_handler: PartitionHandler,
}

impl<S: Store + Clone> QueryExecutor<S> {
    pub fn new(cfg: Config, transport: Sender<IngesterRequest>, store: S) -> QueryExecutor<S> {
        let path = cfg.dir.clone();
        QueryExecutor {
            store: store,
            cfg: cfg,
            ingester_transport: transport,
            partition_handler: PartitionHandler {
                partition_path: path,
            },
        }
    }

    pub fn execute(
        &mut self,
        query: String,
        start_ts: u64,
        end_ts: u64,
        forward: bool,
        limit: u64,
    ) -> Result<String, failure::Error> {
        let mut query = parser::parse(query)?;

        // We'll give more precedence for request limit.
        if limit != 0 {
            query.limit = limit;
        }
        let mut itrs = Vec::new();
        let mut partitions = query.soruces;
        // If there is no partitions mentioned then we have to query all
        // the partitions.
        if partitions.len() == 0 {
            let result = self.partition_handler.partitions();
            if result.is_err() {
                return Err(format_err!("{}", result.unwrap_err()));
            }
            partitions = result.unwrap().partitions;
        }
        for partition in partitions {
            // let the ingester know that we may need the segments. So, sending
            // hint to the ingester so that it'll flush the buffered segement :)
            // Do you hava any smart way? pls do it here.

            let (complete_sender, complete_receiver) = oneshot::channel();
            let hint = IngesterFlushHintReq {
                app: partition.clone(),
                start_ts: start_ts,
                end_ts: end_ts,
                complete_signal: complete_sender,
            };
            block_on(async {
                self.ingester_transport
                    .send(IngesterRequest::Flush(hint))
                    .await
                    .unwrap();
                if let Err(e) = complete_receiver.await {
                    return Err(format_err!("{}", e));
                }
                Ok(())
            })?;

            // no need to copy store every time. we can do the partition registry
            // retrival here. we can replace, if it is show in the profiles.
            let itr = PartitionIterator::new(
                partition,
                start_ts,
                end_ts,
                query.selection.clone(),
                self.store.clone(),
                self.cfg.clone(),
                !forward,
            );
            // some error happened.
            if let Err(e) = itr {
                return Err(format_err!("{}", e));
            }
            let itr = itr.unwrap();
            // no segement files for the given partition.
            if let None = itr {
                continue;
            }
            itrs.push(Rc::new(RefCell::new(itr.unwrap())));
        }
        let itr = MergeIteartor::new(itrs, !forward);
        if let Err(e) = itr {
            return Err(format_err!("{}", e));
        }
        let mut itr = itr.unwrap();
        // We got all the log lines. from here we'll execute the query
        // and parse the json.

        if let Some(distinct) = query.distinct {
            // Find the distinct values for the given attribute.
            let distinct_map = self.handle_distinct(&mut itr, &distinct)?;

            // Build json objects.
            let mut objects = Vec::new();
            if distinct.count {
                for (key, val) in distinct_map {
                    let obj = json!({ format!("{}", key): val }).to_string();
                    objects.push(obj);
                }
            } else {
                for (key, _) in distinct_map {
                    objects.push(format!("\"{}\"", key));
                }
            }

            // Build the output json.
            let out = self.build_json(objects);
            return Ok(out);
        }

        if let Some(average) = query.average {
            let average_map = self.handle_average(&mut itr, &average)?;

            // Build object json
            let objects =
                self.build_json_objects_for_group_by(average_map, average.by, average.alias, 0.0);

            let out = self.build_json(objects);
            return Ok(out);
        }

        if let Some(count) = query.count {
            let count_map = self.handle_count(&mut itr, &count)?;

            // Build object json
            let objects = self.build_json_objects_for_group_by(count_map, count.by, count.alias, 0);

            let out = self.build_json(objects);
            return Ok(out);
        }
        // TODO: add support for offset.
        let mut lines = Vec::new();
        loop {
            match itr.entry() {
                None => break,
                Some(entry) => {
                    if query.limit != 0 && (query.limit as usize) <= lines.len() {
                        break;
                    }
                    lines.push(ResLine {
                        ts: entry.ts,
                        line: String::from_utf8(entry.line.clone()).unwrap(),
                        app: itr.partition().clone(),
                    });
                    itr.next()?;
                }
            }
        }
        Ok(self.build_json(self.build_json_object_for_entry(lines)))
    }

    fn build_json_object_for_entry(&self, entries: Vec<ResLine>) -> Vec<String> {
        let mut objects = Vec::new();

        // build each objects
        for entry in entries {
            let object = format!(
                "{{\"ts\":{}, \"entry\":{}, \"source\":\"{}\" }}",
                entry.ts, entry.line, entry.app,
            );
            objects.push(object);
        }
        objects
    }

    /// build_json_objects_for_group_by creates json objects for group by attr.
    fn build_json_objects_for_group_by<T: std::fmt::Display>(
        &self,
        mut map: HashMap<String, T>,
        by: Option<String>,
        alias: String,
        default: T,
    ) -> Vec<String> {
        let mut objects = Vec::with_capacity(map.len());

        if let Some(group_by_attr) = by {
            for (key, val) in map.drain() {
                objects.push(
                    json!(
                        { format!("{}", &alias): format!("{}", val) ,
                        format!("{}", &group_by_attr): key,
                        }
                    )
                    .to_string(),
                );
            }
            if objects.len() == 0 {
                // Fill the empty object.
                objects.push(
                    json!(
                        { format!("{}", &alias): "".to_string() ,
                        format!("{}", &group_by_attr): "".to_string(),
                        }
                    )
                    .to_string(),
                );
            }
        } else {
            let val = map
                .remove(&"default".to_string())
                .unwrap_or_else(|| default);
            objects.push(
                json!({
                    format!("{}", &alias): format!("{}",val),
                })
                .to_string(),
            );
        };
        objects
    }

    /// build json is used to build output json from the given vector of jsons.
    fn build_json(&self, mut objects: Vec<String>) -> String {
        let mut out = String::from(
            r#"{
            "data":[
        "#,
        );
        for i in 0..objects.len() {
            out.push_str(&mut objects[i]);
            if i < objects.len() - 1 {
                out.push_str(&mut r#","#);
            }
        }
        out.push_str(&mut r#"] }"#);
        out
    }

    fn handle_count(
        &self,
        itr: &mut MergeIteartor<S>,
        count: &parser::Count,
    ) -> Result<HashMap<String, u64>, failure::Error> {
        let key_for_count = &count.attr;

        let mut count_map: HashMap<String, u64> = HashMap::default();
        // I kind of feel there is a over lap between distinct count and count.
        // Need to fix more on it. Need to find a person who could help us in
        // creating proper grammar for us.
        let mut increment_distinct = |key: String| {
            if let Some(val) = count_map.get_mut(&key) {
                *val = *val + 1;
                return;
            }
            count_map.insert(key, 1);
        };
        self.loop_over_iterator(itr, |entry| {
            match get_value_from_json(key_for_count.clone(), &mut entry.line.clone()) {
                Ok(val) => {
                    if val.is_none() {
                        // Don't count just skip.
                        return Ok(());
                    }
                    if let Some(group_by_attr) = count.by.as_ref() {
                        get_value_from_json(group_by_attr.clone(), &mut entry.line.clone())
                            .map(|val| {
                                if val.is_none() {
                                    increment_distinct("\"\"".to_string());
                                    return;
                                }
                                let val = val.unwrap();
                                match val {
                                    Value::String(key) => {
                                        increment_distinct(key.into_owned());
                                    }
                                    Value::Static(node) => {
                                        let key = convert_static_node_to_string(node)
                                            .unwrap_or("\"\"".to_string());
                                        increment_distinct(key);
                                    }
                                    _ => {
                                        increment_distinct("\"\"".to_string());
                                    }
                                }
                            })
                            .map_err(|e| {
                                warn!("unable to parse json {}", e);
                                increment_distinct("\"\"".to_string());
                            })
                            .ok();
                        return Ok(());
                    }
                    increment_distinct("default".to_string());
                }
                Err(e) => {
                    warn!("unable to parse json {}", e);
                }
            }
            return Ok(());
        })?;

        Ok(count_map)
    }

    /// handle distinct gives the distinct count of the given log lines based on the query attr.
    fn handle_distinct(
        &self,
        itr: &mut MergeIteartor<S>,
        distinct: &parser::Distinct,
    ) -> Result<HashMap<String, u64>, failure::Error> {
        let key_for_distinct_lookup = &distinct.attr;
        let mut distinct_map: HashMap<String, u64> = HashMap::default();
        self.loop_over_iterator(itr, |entry| {
            // Ingore all the unstructred logs.
            if entry.structured != STRUCTURED_DATA {
                return Ok(());
            }
            // Get the value of the given json key and insert to hashmap to get
            // the distinct value.
            // TODO: avoid cloning. next should borrow value instead of Rc. But, I'm
            // keeping this way, to know more how the query layer is going to evolve.
            // Time being let's make it work. In future, if we end up in more memory
            // allocation we can optimize here. :)
            match get_value_from_json(key_for_distinct_lookup.clone(), &mut entry.line.clone()) {
                Err(e) => {
                    // Let's not return error. Just log it here.
                    warn!("unable to parse json {}", e);
                    return Ok(());
                }
                Ok(result) => {
                    let mut count_distinct = |key: String| {
                        if let Some(val) = distinct_map.get_mut(&key) {
                            *val = *val + 1;
                            return;
                        }
                        distinct_map.insert(key, 1);
                    };
                    if let Some(val) = result {
                        match val {
                            Value::String(key) => {
                                let key = key.into_owned();
                                count_distinct(key);
                            }
                            Value::Static(node) => {
                                if let Some(key) = convert_static_node_to_string(node) {
                                    count_distinct(key);
                                }
                            }
                            _ => {
                                // We don't count
                            }
                        }
                    }
                    return Ok(());
                }
            }
        })?;
        Ok(distinct_map)
    }

    fn handle_average(
        &self,
        itr: &mut MergeIteartor<S>,
        average: &parser::Average,
    ) -> Result<(HashMap<String, f32>), failure::Error> {
        let key_for_average_lookup = &average.attr;
        let group_by = average.by.as_ref();

        let mut distinct_map: HashMap<String, Vec<f32>> = HashMap::default();

        let mut distict_collector = |key: String, val: f32| {
            if let Some(bucket) = distinct_map.get_mut(&key) {
                bucket.push(val);
                return;
            }
            distinct_map.insert(key, vec![val]);
        };

        //Iterate over entries and find the average of the given key.
        if let Err(e) = self.loop_over_iterator(itr, |entry| {
            // Skip unstructed data.
            if entry.structured != STRUCTURED_DATA {
                return Ok(());
            }

            match get_value_from_json(key_for_average_lookup.clone(), &mut entry.line.clone()) {
                Err(e) => {
                    warn!("Unable to parse json {}", e);
                    return Ok(());
                }
                Ok(val) => {
                    if val.is_none() {
                        return Ok(());
                    }
                    let val = val.unwrap();
                    let val = match val {
                        Value::String(val) => convert_string_to_f32(val.into_owned()),
                        Value::Static(node) => convert_static_node_to_f32(node),
                        _ => None,
                    };
                    // If there is no val just skip it go for further iteration.
                    if val.is_none() {
                        return Ok(());
                    }
                    let val = val.unwrap();
                    // check whether there is any group by
                    if let Some(group_by_attr) = group_by {
                        match get_value_from_json(group_by_attr.clone(), &mut entry.line.clone()) {
                            Ok(group_by_val) => match group_by_val {
                                Some(group_by_val) => match group_by_val {
                                    Value::String(key) => {
                                        distict_collector(key.into_owned(), val);
                                    }
                                    Value::Static(node) => {
                                        let key = convert_static_node_to_string(node)
                                            .unwrap_or_else(|| "\"\"".to_string());
                                        distict_collector(key, val);
                                    }
                                    _ => {
                                        distict_collector("\"\"".to_string(), val);
                                    }
                                },
                                None => {
                                    distict_collector("\"\"".to_string(), val);
                                }
                            },
                            Err(e) => {
                                warn!("unable to parse json {}", e);
                                distict_collector("\"\"".to_string(), val);
                            }
                        }
                        return Ok(());
                    }
                    distict_collector("default".to_string(), val);
                }
            }
            Ok(())
        }) {
            return Err(e);
        }
        let mut out = HashMap::with_capacity(distinct_map.len());

        for (key, bucket) in distinct_map.drain() {
            // Calculate average for each bucket.
            let total: f32 = Iterator::sum(bucket.iter());
            let average = total / bucket.len() as f32;
            out.insert(key, average);
        }
        Ok(out)
    }

    /// loop over iterator is used to iterate over all the entries of the
    /// given iterator and the each entry is passed to the given callback.
    fn loop_over_iterator<'a, F>(
        &self,
        itr: &mut MergeIteartor<S>,
        mut cb: F,
    ) -> Result<(), failure::Error>
    where
        F: FnMut(Rc<Entry>) -> Result<(), failure::Error>,
    {
        loop {
            match itr.entry() {
                None => break,
                Some(entry) => {
                    cb(entry)?;
                    itr.next()?;
                }
            }
        }
        Ok(())
    }
}

impl<S: Store + Clone> Clone for QueryExecutor<S> {
    fn clone(&self) -> QueryExecutor<S> {
        QueryExecutor {
            store: self.store.clone(),
            cfg: self.cfg.clone(),
            ingester_transport: self.ingester_transport.clone(),
            partition_handler: self.partition_handler.clone(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::config::config::Config;
    use crate::iterator::merge_iterator::MergeIteartor;
    use crate::parser::*;
    use crate::partition::partition_iterator::PartitionIterator;
    use crate::partition::segment_writer::tests::{get_test_cfg, get_test_store};
    use crate::partition::segment_writer::SegmentWriter;
    use crate::store::rocks_store::RocksStore;
    use crate::types::types::api::PushLogLine;
    use futures::channel::mpsc;
    use std::cell::RefCell;
    use std::rc::Rc;

    // get_merge_iterator returns merge iterator fot the given log lines.
    fn get_merge_iterator(
        cfg: Config,
        store: RocksStore,
        lines: Vec<PushLogLine>,
    ) -> MergeIteartor<RocksStore> {
        let mut segment_writer = SegmentWriter::new(
            cfg.clone(),
            String::from("tmppartition"),
            1,
            store.clone(),
            2,
        )
        .unwrap();
        segment_writer.push(lines).unwrap();
        segment_writer.flush().unwrap();
        segment_writer.close();

        // Create iterator for the given segment.
        let iterator = PartitionIterator::new(
            String::from("tmppartition"),
            1,
            9,
            None,
            store.clone(),
            cfg.clone(),
            false,
        )
        .unwrap();
        MergeIteartor::new(vec![Rc::new(RefCell::new(iterator.unwrap()))], false).unwrap()
    }
    // Test distinct
    #[test]
    fn test_distinct() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone()).clone();
        // Create log lines.
        let mut lines = Vec::new();
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            }
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 2,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Go",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            }
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 3,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Ruby",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            }
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 4,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            }
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 5,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Go",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            }
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 7,
            structured: true,
            json_keys: Vec::default(),
        });

        // create merge iterator for the log lines.
        let mut itr = get_merge_iterator(cfg.clone(), store.clone(), lines);

        // create dummy transport for the executor
        let (sender, _) = mpsc::channel(1000);
        // Create executor to test the query.
        let mut executor = QueryExecutor::new(cfg, sender, store);

        let distinct = parser::Distinct {
            attr: "skills.language".to_string(),
            alias: "languages".to_string(),
            count: false,
        };

        let distinct_map = executor.handle_distinct(&mut itr, &distinct).unwrap();

        assert_eq!(distinct_map.len(), 3);

        assert_eq!(distinct_map.get("Go").is_some(), true);
        assert_eq!(distinct_map.get("Rust").is_some(), true);
        assert_eq!(distinct_map.get("Ruby").is_some(), true);
        assert_eq!(*distinct_map.get("Go").unwrap(), 2);
        assert_eq!(*distinct_map.get("Rust").unwrap(), 2);
        assert_eq!(*distinct_map.get("Ruby").unwrap(), 1);
    }

    #[test]
    fn test_average() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone()).clone();
        // Create log lines.
        let mut lines = Vec::new();
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "la": ["yo","yo1"]
            },
            "age": 30
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 2,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Go",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 40
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 3,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Ruby",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 20
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 4,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 15
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 5,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Go",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 45
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 7,
            structured: true,
            json_keys: Vec::default(),
        });

        // create merge iterator for the log lines.
        let mut itr = get_merge_iterator(cfg.clone(), store.clone(), lines);

        // create dummy transport for the executor
        let (sender, _) = mpsc::channel(1000);
        // Create executor to test the query.
        let executor = QueryExecutor::new(cfg, sender, store);

        let average = parser::Average {
            attr: "age".to_string(),
            alias: "average_age".to_string(),
            by: None,
        };

        let out = executor.handle_average(&mut itr, &average).unwrap();
        assert_eq!(*out.get("default").unwrap(), 30.0);
    }

    #[test]
    fn test_average_group_by() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone()).clone();
        // Create log lines.
        let mut lines = Vec::new();
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "la": ["yo","yo1"]
            },
            "age": 30
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 2,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Go",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 40
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 3,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Ruby",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 20
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 4,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 15
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 5,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Go",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 45
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 7,
            structured: true,
            json_keys: Vec::default(),
        });

        // create merge iterator for the log lines.
        let mut itr = get_merge_iterator(cfg.clone(), store.clone(), lines);

        // create dummy transport for the executor
        let (sender, _) = mpsc::channel(1000);
        // Create executor to test the query.
        let executor = QueryExecutor::new(cfg, sender, store);

        let average = parser::Average {
            attr: "age".to_string(),
            alias: "average_age".to_string(),
            by: Some("skills.language".to_string()),
        };

        let out = executor.handle_average(&mut itr, &average).unwrap();
        assert_eq!(*out.get(&"Ruby".to_string()).unwrap(), 20.0);
        assert_eq!(*out.get(&"Rust".to_string()).unwrap(), 22.5);
        assert_eq!(*out.get(&"Go".to_string()).unwrap(), 42.5);
    }

    #[test]
    fn test_count() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone()).clone();
        // Create log lines.
        let mut lines = Vec::new();
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "la": ["yo","yo1"]
            },
            "age": 30
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 2,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Go",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 40
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 3,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Ruby",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 20
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 4,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 15
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 5,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Go",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 45
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 7,
            structured: true,
            json_keys: Vec::default(),
        });

        // create merge iterator for the log lines.
        let mut itr = get_merge_iterator(cfg.clone(), store.clone(), lines);

        // create dummy transport for the executor
        let (sender, _) = mpsc::channel(1000);
        // Create executor to test the query.
        let executor = QueryExecutor::new(cfg, sender, store);

        let count = parser::Count {
            attr: "age".to_string(),
            alias: "age_count".to_string(),
            by: None,
        };

        let count_map = executor.handle_count(&mut itr, &count).unwrap();
        assert_eq!(*count_map.get(&"default".to_string()).unwrap(), 5);
    }

    #[test]
    fn test_count_by() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone()).clone();
        // Create log lines.
        let mut lines = Vec::new();
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "la": ["yo","yo1"]
            },
            "age": 30
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 2,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Go",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 40
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 3,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Ruby",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 20
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 4,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Rust",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 15
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 5,
            structured: true,
            json_keys: Vec::default(),
        });
        lines.push(PushLogLine {
            raw_data: br#"{
            "name": "Licenser",
            "skills": {
                "language": "Go",
                "yo": {
                    "age": 1
                },
                "la": ["yo","yo1"]
            },
            "age": 45
        }"#
            .to_vec(),
            indexes: Vec::new(),
            ts: 7,
            structured: true,
            json_keys: Vec::default(),
        });

        // create merge iterator for the log lines.
        let mut itr = get_merge_iterator(cfg.clone(), store.clone(), lines);

        // create dummy transport for the executor
        let (sender, _) = mpsc::channel(1000);
        // Create executor to test the query.
        let executor = QueryExecutor::new(cfg, sender, store);

        let count = parser::Count {
            attr: "age".to_string(),
            alias: "age_count".to_string(),
            by: Some("skills.language".to_string()),
        };

        let count_map = executor.handle_count(&mut itr, &count).unwrap();
        assert_eq!(*count_map.get(&"Rust".to_string()).unwrap(), 2);
        assert_eq!(*count_map.get(&"Ruby".to_string()).unwrap(), 1);
        assert_eq!(*count_map.get(&"Go".to_string()).unwrap(), 2);
    }
}
