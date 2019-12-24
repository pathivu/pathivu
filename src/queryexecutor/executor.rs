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
use crate::store::batch::Batch;
use crate::store::store::Store;
use crate::types::types::{IngesterFlushHintReq, IngesterRequest};
use crate::types::types::{QueryRequest, QueryResponse, ResLine};
use failure::format_err;
use futures::channel::mpsc::Sender;
use futures::channel::oneshot;
use futures::executor::block_on;
use futures::sink::SinkExt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
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
    // execute will excute the query and give back the response.
    pub fn execute(&mut self, req: QueryRequest) -> Result<QueryResponse, String> {
        let mut itrs = Vec::new();
        let mut partitions = req.partitions;
        // If there is no partitions mentioned then we have to query all
        // the partitions.
        if partitions.len() == 0 {
            let result = self.partition_handler.partitions();
            if result.is_err() {
                return Err(format!("{}", result.unwrap_err()));
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
                start_ts: req.start_ts,
                end_ts: req.end_ts,
                complete_signal: complete_sender,
            };
            block_on(async {
                self.ingester_transport
                    .send(IngesterRequest::Flush(hint))
                    .await
                    .unwrap();
                if let Err(e) = complete_receiver.await {
                    return Err(format!("{}", e));
                }
                Ok(())
            })?;

            // no need to copy store every time. we can do the partition registry
            // retrival here. we can replace, if it is show in the profiles.
            let itr = PartitionIterator::new(
                partition,
                req.start_ts,
                req.end_ts,
                None,
                self.store.clone(),
                self.cfg.clone(),
                !req.forward,
            );
            // some error happened.
            if let Err(e) = itr {
                return Err(format!("{}", e));
            }
            let itr = itr.unwrap();
            // no segement files for the given partition.
            if let None = itr {
                continue;
            }
            itrs.push(Rc::new(RefCell::new(itr.unwrap())));
        }
        let itr = MergeIteartor::new(itrs, !req.forward);
        if let Err(e) = itr {
            return Err(format!("{}", e));
        }
        let mut itr = itr.unwrap();
        // TODO: add support for offset.
        let mut lines = Vec::new();
        loop {
            match itr.entry() {
                None => break,
                Some(entry) => {
                    lines.push(ResLine {
                        line: String::from_utf8(entry.line.clone()).unwrap(),
                        ts: entry.ts,
                        app: itr.partition().clone(),
                    });
                    if let Err(e) = itr.next() {
                        return Err(format!("{}", e));
                    }
                }
            }
        }
        Ok(QueryResponse { lines: lines })
    }

    pub fn execute_sql_query(
        &mut self,
        query: String,
        start_ts: u64,
        end_ts: u64,
        forward: bool,
    ) -> Result<(), failure::Error> {
        let query = parser::parse(query)?;
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
                None,
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
        }
        Ok(())
    }

    fn handle_distinct(&self, itr: &MergeIteartor<S>, distinct: parser::Distinct) {
        let distinct_map = HashMap::new();
        let key_for_distinct_lookup = distinct.attr;

        let cb = |distinct_map: &HashMap<String, u64>| {
            return |entry: Rc<Entry>| {
                // Ingore all the unstructred logs.
                if entry.structured != 1 {
                    return;
                }
                match get_value_from_json(key_for_distinct_lookup.clone(), &mut entry.line) {
                    Err(e) => {}
                }
            };
        };
    }
    /// loop over iterator is used to iterate over all the entries of the
    /// given iterator and the each entry is passed to the given callback.
    fn loop_over_iterator(
        &self,
        itr: &MergeIteartor<S>,
        cb: Box<dyn Fn(Rc<Entry>)>,
    ) -> Result<(), failure::Error> {
        loop {
            match itr.entry() {
                None => break,
                Some(entry) => {
                    cb(entry);
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
