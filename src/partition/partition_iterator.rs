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
use crate::partition::iterator::Iterator;
use crate::partition::segment_iterator::Entry;
use crate::partition::segment_iterator::SegmentIterator;
use crate::store::batch::Batch;
use crate::store::store::Store;
use crate::types::types::{PartitionRegistry, SegmentFile, PARTITION_PREFIX};
use failure;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;

/// Paritition iterator is used to iterate over all the segment iterator
/// of the given partition.
pub struct PartitionIterator<S> {
    store: S,
    current_segment: usize,
    segment_files: Vec<SegmentFile>,
    current_iterator: Option<SegmentIterator<S>>,
    cfg: Config,
    partition: String,
    query: String,
    start_ts: u64,
    end_ts: u64,
    backward: bool,
}

impl<S: Store + Clone> PartitionIterator<S> {
    /// constructor for partition iterator.
    pub fn new(
        partition: String,
        start_ts: u64,
        end_ts: u64,
        query: String,
        store: S,
        cfg: Config,
        backward: bool,
    ) -> Result<Option<PartitionIterator<S>>, failure::Error> {
        let buf = store.get(format!("{}_{}", PARTITION_PREFIX, partition).as_bytes())?;
        if buf.is_none() {
            // This case can happen if there is no segment files has been flushed.
            return Ok(None);
        }
        let buf = buf.unwrap();
        let mut buf = Deserializer::new(&buf[..]);
        let registry: PartitionRegistry = Deserialize::deserialize(&mut buf)?;
        // TODO: make it binary search.
        let mut filtered_segments = Vec::new();
        for segment_file in registry.segment_files {
            // we'll filter the segment files, which lies between the given time stamo
            if (segment_file.start_ts >= start_ts && segment_file.start_ts <= end_ts)
                || (segment_file.end_ts >= start_ts && segment_file.end_ts <= end_ts)
                || (start_ts == 0 && end_ts == 0)
            {
                filtered_segments.push(segment_file.clone());
            }
        }
        if backward {
            filtered_segments.reverse();
        } else {
            filtered_segments.sort();
        }

        let mut current_iterator = None;
        if filtered_segments.len() > 0 {
            let segment_file = filtered_segments.get(0).unwrap();
            let partition_path = Path::new(&cfg.dir).join("partition").join(&partition);
            let iterator = SegmentIterator::new(
                segment_file.id,
                partition_path,
                store.clone(),
                query.clone(),
                partition.clone(),
                start_ts,
                end_ts,
                backward,
            )?;
            current_iterator = Some(iterator);
        }
        Ok(Some(PartitionIterator {
            store: store,
            current_segment: 0,
            segment_files: filtered_segments,
            cfg: cfg,
            current_iterator: current_iterator,
            partition: partition,
            query: query,
            start_ts: start_ts,
            end_ts: end_ts,
            backward: backward,
        }))
    }

    /// returns the current entry of the iterator. If it is none, then there
    /// is no entry left to advance.
    pub fn entry(&self) -> Option<Rc<Entry>> {
        match &self.current_iterator {
            Some(iterator) => return iterator.entry(),
            None => return None,
        }
    }

    /// next will advance the iterator for the next entry. if it is none there is
    /// no entry to advance.
    pub fn next(&mut self) -> Result<Option<()>, failure::Error> {
        match &mut self.current_iterator {
            Some(iterator) => {
                let val = iterator.next();
                if val.is_some() {
                    return Ok(Some(()));
                }
                return self.advance_iteartor();
            }
            None => {
                return self.advance_iteartor();
            }
        }
    }

    /// advance_iterator is used to advance next segment iterator.
    fn advance_iteartor(&mut self) -> Result<Option<()>, failure::Error> {
        if self.current_segment >= self.segment_files.len() - 1 {
            return Ok(None);
        }
        self.current_segment = self.current_segment + 1;
        let partition_path = Path::new(&self.cfg.dir)
            .join("partition")
            .join(&self.partition);
        let segment_file = self.segment_files.get(self.current_segment).unwrap();
        let iterator = SegmentIterator::new(
            segment_file.id,
            partition_path,
            self.store.clone(),
            self.query.clone(),
            self.partition.clone(),
            self.start_ts,
            self.end_ts,
            self.backward,
        )?;
        if iterator.entry().is_none() {
            // this iterator doesn't have the query so recursively advance.
            return self.advance_iteartor();
        }
        self.current_iterator = Some(iterator);
        Ok(Some(()))
    }

    pub fn partition(&self) -> &String {
        return &self.partition;
    }
}

#[cfg(test)]
pub mod tests {
    use crate::config::config::Config;
    use crate::partition::partition_iterator::PartitionIterator;
    use crate::partition::segment_writer::tests::{get_test_cfg, get_test_store};
    use crate::partition::segment_writer::SegmentWriter;
    use crate::store::rocks_store::RocksStore;
    use crate::store::store::Store;
    use crate::types::types::{LogLine, PartitionRegistry, PARTITION_PREFIX};
    pub fn create_segment(
        id: u64,
        partition: String,
        cfg: Config,
        start_ts: u64,
        store: RocksStore,
    ) {
        let mut segment_writer = SegmentWriter::new(cfg, partition, id, store, start_ts).unwrap();
        let mut lines = Vec::new();
        lines.push(LogLine {
            line: format!("liala transfered {} money to raja", start_ts + 100),
            indexes: vec![
                "liala".to_string(),
                "transfered".to_string(),
                "money".to_string(),
                "raja".to_string(),
            ],
            ts: start_ts,
        });
        lines.push(LogLine {
            line: format!("liala transfered {} money to raja", start_ts + 300),
            indexes: vec![
                "roja".to_string(),
                "transfered".to_string(),
                "money".to_string(),
                "navin".to_string(),
            ],
            ts: start_ts + 2,
        });
        segment_writer.push(lines).unwrap();
        segment_writer.close().unwrap();
    }
    #[test]
    fn test_partition_iterator() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone());
        let partition_name = String::from("temppartition");
        create_segment(1, partition_name.clone(), cfg.clone(), 1, store.clone());
        create_segment(2, partition_name.clone(), cfg.clone(), 4, store.clone());
        create_segment(3, partition_name.clone(), cfg.clone(), 7, store.clone());
        create_segment(4, partition_name.clone(), cfg.clone(), 10, store.clone());
        let mut partition_iterator =
            PartitionIterator::new(partition_name, 1, 9, String::from(""), store, cfg, false)
                .unwrap()
                .unwrap();
        assert_eq!(partition_iterator.entry().unwrap().ts, 1);
        partition_iterator.next();
        partition_iterator.next();
        assert_eq!(partition_iterator.entry().unwrap().ts, 4);
        partition_iterator.next();
        assert_eq!(partition_iterator.entry().unwrap().ts, 6);
        partition_iterator.next();
        assert_eq!(partition_iterator.entry().unwrap().ts, 7);
        partition_iterator.next();
        assert_eq!(partition_iterator.entry().unwrap().ts, 9);
        let valid = partition_iterator.next().unwrap();
        assert!(valid.is_none());
        assert!(partition_iterator.entry().is_none());
    }

    #[test]
    fn test_partition_iterator_backward() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone());
        let partition_name = String::from("temppartition");
        create_segment(1, partition_name.clone(), cfg.clone(), 1, store.clone());
        create_segment(2, partition_name.clone(), cfg.clone(), 4, store.clone());
        create_segment(3, partition_name.clone(), cfg.clone(), 7, store.clone());
        create_segment(4, partition_name.clone(), cfg.clone(), 10, store.clone());
        let mut partition_iterator =
            PartitionIterator::new(partition_name, 1, 9, String::from(""), store, cfg, true)
                .unwrap()
                .unwrap();
        assert_eq!(partition_iterator.entry().unwrap().ts, 9);
        partition_iterator.next();
        partition_iterator.next();
        assert_eq!(partition_iterator.entry().unwrap().ts, 6);
        partition_iterator.next();
        assert_eq!(partition_iterator.entry().unwrap().ts, 4);
        partition_iterator.next();
        assert_eq!(partition_iterator.entry().unwrap().ts, 3);
        partition_iterator.next();
        assert_eq!(partition_iterator.entry().unwrap().ts, 1);
        let valid = partition_iterator.next().unwrap();
        assert!(valid.is_none());
        assert!(partition_iterator.entry().is_none());
    }
}
