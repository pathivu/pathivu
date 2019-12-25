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
use crate::store::batch::Batch;
use crate::store::store::Store;
use crate::types::types;
use crate::types::types::{
    LogLine, PartitionRegistry, SegmentFile, PARTITION_PREFIX, POSTING_LIST_ALL,
    SEGEMENT_JSON_KEY_PREFIX, SEGMENT_PREFIX, STRUCTURED_DATA, UN_STRUCTURED_DATA,
};
use byteorder::{LittleEndian, WriteBytesExt};
use failure::{bail, Error};
use fst::raw::Builder;
use fst::{IntoStreamer, Map, SetBuilder, Streamer};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, IoSlice, Write};
use std::mem;
use std::path::Path;
use std::time::Duration;
const MAGIC_TEXT: &str = "Chola!Header!";
const MAGIC_NUMBER: u8 = 1;
use crate::partition::segment_iterator::Entry;
use binary_heap_plus::BinaryHeap;
use binary_heap_plus::MinComparator;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::marker::PhantomData;

/// SegmentWriter is used to write segment files.
type Index = String;

pub struct SegmentWriter<S> {
    pub partition: String,
    config: Config,
    segment_file: File,
    batch: Vec<types::api::PushLogLine>,
    file_offset: u64,
    store: S,
    id: u64,
    start_ts: u64,
    end_ts: u64,
    batch_count: usize,
    index_posting_list: HashMap<String, Vec<u8>>,
    json_key_posting_list: HashMap<String, Vec<u8>>,
    index_size: usize,
    entry_offsets: Vec<u8>,
    flushed_offset: u64,
}

impl<S: Store> SegmentWriter<S> {
    /// new create a segment file and returns SegmentWriter.
    pub fn new(
        config: Config,
        partition: String,
        id: u64,
        store: S,
        start_ts: u64,
    ) -> Result<SegmentWriter<S>, Error> {
        let path = Path::new(&config.dir)
            .join("partition")
            .join(partition.to_string());
        create_dir_all(&path)?;
        // create segment file.
        let mut file = File::create(path.join(format!("{}.segment", id)))?;

        // write the layout.
        let mut buf = Vec::new();
        buf.push(IoSlice::new(&MAGIC_TEXT.as_bytes()));
        buf.push(IoSlice::new(&[MAGIC_NUMBER; 1]));
        let _ = file.write_vectored(&buf[..])?;
        let file_index = (&MAGIC_TEXT.as_bytes().len() + 1) as u64;
        let info = file.metadata().unwrap();
        assert_eq!(info.len(), 14);
        Ok(SegmentWriter {
            config: config,
            segment_file: file,
            partition: partition,
            batch: Vec::new(),
            file_offset: file_index,
            store: store,
            id: id,
            start_ts: start_ts,
            end_ts: start_ts,
            batch_count: 0,
            index_posting_list: HashMap::default(),
            json_key_posting_list: HashMap::default(),
            index_size: 0,
            entry_offsets: Vec::new(),
            flushed_offset: file_index,
        })
    }

    /// push pushes logline and terms of the logline. terms are used to index the line's file
    /// offset. push will essentially batch all the line. Once the batch size reached, it'll
    /// flush the line to the file.
    pub fn push(&mut self, mut lines: Vec<types::api::PushLogLine>) -> Result<(), Error> {
        // encoding line length.
        let drain = lines.drain(0..lines.len());
        for line in drain {
            self.batch.push(line);
        }
        if self.batch.len() >= (self.config.max_batch_size as usize) {
            debug!("flushing log lines for the partition {}", self.partition);
            self.flush()?;
        }
        Ok(())
    }

    /// flush writes all the batched logline to the segment file.
    pub fn flush(&mut self) -> Result<(), Error> {
        // batch all the vectors.
        let mut buf = Vec::new();
        let drain = self.batch.drain(0..self.batch.len());
        for mut log_line in drain {
            // Encode log line offset.
            let mut line_offset = [0u8; mem::size_of::<u64>()];
            line_offset
                .as_mut()
                .write_u64::<LittleEndian>(self.file_offset);
            // Encode log line.
            let line_buf = log_line.raw_data;
            // Encode time stamp.
            let mut ts_buf = [0u8; mem::size_of::<u64>()];
            ts_buf.as_mut().write_u64::<LittleEndian>(log_line.ts);
            let indexes = log_line.indexes.drain(0..log_line.indexes.len());
            // we'll track of all the offsets because If no search given, we should give
            // all the lines.
            self.entry_offsets.extend(line_offset.to_vec());
            for term in indexes {
                self.index_size = self.index_size + term.len();
                if let Some(posting_list) = self.index_posting_list.get_mut(&term) {
                    posting_list.extend(line_offset.to_vec().iter());
                } else {
                    self.index_posting_list
                        .insert(term.clone(), line_offset.to_vec());
                }
            }

            // Populate posting list of json keys as well.
            let json_keys = log_line.json_keys.drain(0..log_line.json_keys.len());
            for key in json_keys {
                if let Some(posting_list) = self.json_key_posting_list.get_mut(&key) {
                    posting_list.extend(line_offset.to_vec().iter());
                } else {
                    self.json_key_posting_list.insert(key, line_offset.to_vec());
                }
            }
            // TS LENGTH + STRUCTURED KEY + LINE LENGTH.
            let entry_length = (line_buf.len() + ts_buf.len() + 1) as u64;
            // advance the file offset.
            self.file_offset = self.file_offset + entry_length + 8;
            // encode entry length.
            let mut len_buf = [0u8; mem::size_of::<u64>()];
            len_buf.as_mut().write_u64::<LittleEndian>(entry_length);
            buf.push(len_buf.to_vec());
            buf.push(ts_buf.to_vec());
            // If it is json set the 1 otherwise set 0.
            if log_line.structured {
                buf.push(vec![STRUCTURED_DATA]);
            } else {
                buf.push(vec![UN_STRUCTURED_DATA]);
            }
            // index this line.
            buf.push(line_buf);
            // advance line ts.
            // assert!(self.end_ts < log_line.ts);
            self.end_ts = log_line.ts;
        }
        let mut io_slices = Vec::new();
        for slice in buf.iter() {
            io_slices.push(IoSlice::new(slice));
        }
        // write them in a single syscall. But we have to verify
        // the performance here.
        self.segment_file.write_vectored(&io_slices)?;
        self.segment_file.flush()?;
        self.batch.clear();
        self.flushed_offset = self.file_offset;
        Ok(())
    }

    /// size of the segment file.
    pub fn size(&self) -> u64 {
        self.file_offset
    }

    /// index_size returns size of index file.
    pub fn index_size(&self) -> usize {
        self.index_size
    }

    /// close will flush all the remaining batched line.
    pub fn close(mut self) -> Result<(), Error> {
        info!("flushing {} partition segment {}", self.partition, self.id);
        if self.batch.len() > 0 {
            &self.flush()?;
        }
        let mut partition_registry: PartitionRegistry;
        let (start_ts, end_ts) = (self.start_ts, self.end_ts);
        let segment_file = SegmentFile {
            id: self.id,
            start_ts: start_ts,
            end_ts: end_ts,
        };
        if segment_file.id == 1 {
            // This is new segment so, create a new partition registry and save it.
            let segment_files = vec![segment_file];
            partition_registry = PartitionRegistry {
                last_assigned: 1,
                segment_files: segment_files,
            };
        } else {
            let registry_buf = self
                .store
                .get(format!("{}_{}", PARTITION_PREFIX, self.partition).as_bytes())?
                .unwrap();
            let mut registry_buf = Deserializer::new(&registry_buf[..]);
            partition_registry = Deserialize::deserialize(&mut registry_buf)?;
            partition_registry.segment_files.push(segment_file);
            partition_registry.last_assigned = self.id;
        }
        let mut buf = Vec::with_capacity(1024);
        partition_registry
            .serialize(&mut Serializer::new(&mut buf))
            .unwrap();
        self.store.set(
            format!("{}_{}", PARTITION_PREFIX, self.partition).as_bytes(),
            buf,
        );
        // insert posting list.
        let mut indices = Vec::new();
        let mut wb = Batch::new();
        let posting_list = self.index_posting_list.drain();
        for (index, list) in posting_list {
            wb.set(
                format!(
                    "{}_{}_{}_{}",
                    SEGMENT_PREFIX, self.partition, self.id, &index
                )
                .into_bytes(),
                list,
            )
            .unwrap();
            indices.push(index);
        }
        // Insert json key posting list.
        let posting_list = self.json_key_posting_list.drain();
        for (key, list) in posting_list {
            wb.set(
                format!(
                    "{}_{}_{}_{}",
                    SEGEMENT_JSON_KEY_PREFIX, self.partition, self.id, &key
                )
                .into_bytes(),
                list,
            )
            .unwrap();
        }
        wb.set(
            format!(
                "{}_{}_{}_{}",
                SEGMENT_PREFIX, self.partition, self.id, POSTING_LIST_ALL,
            )
            .into_bytes(),
            self.entry_offsets,
        )
        .unwrap();
        self.store.flush_batch(wb)?;
        indices.sort();
        let indices = indices.drain(..);
        let mut index_writer = SetBuilder::memory();
        for index in indices {
            index_writer.insert(index).unwrap();
        }
        let mut index_file = File::create(
            Path::new(&self.config.dir)
                .join("partition")
                .join(self.partition.to_string())
                .join(format!("segment_index_{}.fst", self.id)),
        )?;
        let buf = index_writer.into_inner().unwrap();
        index_file.write(&buf)?;
        index_file.flush()?;
        let info = self.segment_file.metadata().unwrap();
        assert_eq!(info.len(), self.file_offset);
        Ok(())
    }

    pub fn file_id(&self) -> u64 {
        self.id
    }

    /// segment_ts givens start ts and end ts of the segment
    pub fn segment_ts(&self) -> (u64, u64) {
        (self.start_ts, self.end_ts)
    }

    pub fn get_inmemory_hint(
        &self,
        query: String,
        start_ts: u64,
        end_ts: u64,
    ) -> Option<Vec<Entry>> {
        if self.end_ts > start_ts {
            return None;
        }
        return None;
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::config::config::Config;
    use crate::parser::parser::Selection;
    use crate::partition::iterator::Iterator;
    use crate::partition::segment_iterator::SegmentIterator;
    use crate::store::rocks_store::RocksStore;
    use crate::types::types::api::PushLogLine;
    use crate::types::types::LogLine;
    use std::path::Path;
    use tempfile;
    pub fn get_test_cfg() -> Config {
        let tmp_dir = tempfile::tempdir().unwrap();
        Config {
            dir: tmp_dir.path().to_str().unwrap().to_string(),
            max_segment_size: 100 << 10,
            max_index_size: 100 << 10,
            max_batch_size: 1,
        }
    }
    pub fn get_test_store(cfg: Config) -> RocksStore {
        RocksStore::new(cfg).unwrap()
    }
    #[test]
    fn test_segement_writer_and_itearator() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone());
        let mut segment_writer = SegmentWriter::new(
            cfg.clone(),
            String::from("tmppartition"),
            1,
            store.clone(),
            2,
        )
        .unwrap();
        let mut lines = Vec::new();
        lines.push(PushLogLine {
            raw_data: String::from("liala transfered money to raja").into_bytes(),
            indexes: vec![
                "liala".to_string(),
                "transfered".to_string(),
                "money".to_string(),
                "raja".to_string(),
            ],
            ts: 2,
            structured: false,
            json_keys: Vec::new(),
        });
        lines.push(PushLogLine {
            raw_data: String::from("roja transfered money to navin").into_bytes(),
            indexes: vec![
                "roja".to_string(),
                "transfered".to_string(),
                "money".to_string(),
                "navin".to_string(),
            ],
            ts: 4,
            structured: false,
            json_keys: Vec::new(),
        });
        segment_writer.push(lines);
        // check the end ts.
        assert_eq!(segment_writer.end_ts, 4);
        // check all the entry offset.
        assert_eq!(segment_writer.entry_offsets.len(), 16);
        // check posint list length.
        assert_eq!(segment_writer.index_posting_list.len(), 6);
        segment_writer.close().unwrap();
        let partition_path = Path::new(&cfg.dir).join("partition").join("tmppartition");
        let mut iterator = SegmentIterator::new(
            1,
            partition_path.clone(),
            store.clone(),
            None,
            String::from("tmppartition"),
            1,
            5,
            false,
        )
        .unwrap();
        assert_eq!(iterator.entries.len(), 2);
        assert_eq!(iterator.entries[0].ts, 2);
        assert_eq!(iterator.entries[1].ts, 4);
        let valid = iterator.next();
        assert!(valid.is_some());
        assert_eq!(iterator.entry().unwrap().ts, 4);
        assert_eq!(iterator.current_index, 1);
        let valid = iterator.next();
        assert!(valid.is_none());
        assert_eq!(iterator.current_index, 2);
        let iterator = SegmentIterator::new(
            1,
            partition_path,
            store,
            Some(Selection {
                value: String::from("navi"),
                attr: None,
                structured: false,
            }),
            String::from("tmppartition"),
            1,
            5,
            false,
        )
        .unwrap();
        assert_eq!(iterator.entries.len(), 1);
    }
}
