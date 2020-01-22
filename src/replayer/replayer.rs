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
use crate::replayer::iterator::Iterator;
use crate::store::batch::Batch;
use crate::store::store::Store;
use crate::types::types::{
    PartitionRegistry, SegmentFile, PARTITION_PREFIX, POSTING_LIST_ALL, SEGMENT_PREFIX,
};
use byteorder::{LittleEndian, WriteBytesExt};
use failure::Error;
use fst::SetBuilder;
use rmp_serde::{Deserializer, Serializer};
use rust_stemmers::{Algorithm, Stemmer};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::fs::{create_dir_all, read_dir, remove_file, File};
use std::io::Write;
use std::mem;
use std::path::{Path, PathBuf};
use stopwords::{Language, Spark, Stopwords};
/// Replayer will help us to build the index file and posting list, if index file is not
/// persisted due to crash.
pub struct Replayer<'a, S> {
    cfg: Config,
    stops: HashSet<&'a &'a str>,
    store: S,
    stemmer: Stemmer,
}

impl<'a, S: Store> Replayer<'a, S> {
    pub fn new(cfg: Config, store: S) -> Replayer<'a, S> {
        Replayer {
            cfg: cfg,
            stops: Spark::stopwords(Language::English)
                .unwrap()
                .iter()
                .collect(),
            store: store,
            stemmer: Stemmer::create(Algorithm::English),
        }
    }
    /// replay function replay the crashed segment files.
    pub fn replay(&mut self) -> Result<(), Error> {
        let partition_path = Path::new(&self.cfg.dir).join("partition");
        create_dir_all(&partition_path)?;

        // iterate over every partition and find the crashed segment
        // files.
        for partition in read_dir(&partition_path)? {
            let partition = partition?;
            if partition.path().is_dir() {
                self.replay_segments(partition.path())?;
            }
        }
        Ok(())
    }

    /// replay_segments will check whether segment file have
    pub fn replay_segments(&mut self, partition_path: PathBuf) -> Result<(), Error> {
        let mut fsa_files = HashSet::new();
        let mut segment_ids = Vec::new();
        let partition_name = partition_path
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();

        for entry in read_dir(&partition_path)? {
            let entry = entry?;
            let entry_path = entry.path();
            assert_eq!(entry_path.is_file(), true);
            // check whether it is fsa or segment file.
            let extension = entry_path.extension().unwrap();
            if extension == "fst" {
                // There may be case where fsa file is created but
                // index is not flushed. we need some mechanism to detect
                // that or we could just check the size and build index
                // from the begining.
                // TODO: @balajijinnah.
                let file_name = entry_path.file_stem().unwrap();
                let splits = file_name.to_str().unwrap().split("_");
                let splits = splits.collect::<Vec<&str>>();
                fsa_files.insert(splits[2].to_string());
                continue;
            }
            assert_eq!(extension, "segment");
            let file_name = entry_path.file_stem().unwrap();
            segment_ids.push(file_name.to_str().unwrap().parse::<u64>().unwrap());
        }
        // reverse it because only last file doesn't have partition.
        segment_ids.reverse();
        for segment_id in segment_ids {
            if !fsa_files.contains(&segment_id.to_string()) {
                // this segment file doesn't have index file, so
                // rebuild the index file.
                self.build_fst(&partition_path, segment_id, &partition_name)?;
            }
        }
        Ok(())
    }

    fn build_fst(
        &mut self,
        partition_path: &PathBuf,
        segment_id: u64,
        partition: &String,
    ) -> Result<(), Error> {
        let segment_path = partition_path.join(format!("{}.segment", segment_id));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            //  .truncate(true)
            .open(&segment_path)?;
        let mut entry_offsets: Vec<u8> = Vec::new();
        let mut posting_list: HashMap<String, Vec<u8>> = HashMap::new();
        let mut indices = Vec::new();
        match Iterator::new(&file) {
            Ok(mut itr) => {
                let mut start_ts: u64 = 0;
                let mut end_ts: u64 = 0;
                loop {
                    let item = itr.next()?;
                    match item {
                        Some(entry) => {
                            // set start timestamp and end timestamp.
                            if entry.ts > end_ts {
                                end_ts = entry.ts;
                            }
                            if start_ts == 0 {
                                start_ts = entry.ts;
                            }
                            let line = String::from_utf8(entry.line.clone()).unwrap();
                            let splits = line.split(" ");
                            let mut offset_buf = [0u8; mem::size_of::<u64>()];
                            offset_buf
                                .as_mut()
                                .write_u64::<LittleEndian>(itr.valid_entry_offset())
                                .unwrap();
                            entry_offsets.extend(offset_buf.clone().to_vec());
                            for split in splits {
                                if self.stops.contains(&split) {
                                    continue;
                                }
                                let term = self.stemmer.stem(&split);
                                let term = String::from(term);
                                if let Some(posting_list) = posting_list.get_mut(&term) {
                                    posting_list.extend(offset_buf.to_vec().iter());
                                } else {
                                    posting_list.insert(term.clone(), offset_buf.to_vec());
                                }
                                indices.push(term);
                            }
                        }
                        None => {
                            if entry_offsets.len() == 0 {
                                drop(file);
                                remove_file(segment_path)?;
                                break;
                            }
                            file.set_len(itr.valid_offset())?;
                            // now save the posting list in the store.
                            let mut wb = Batch::new();
                            for (index, list) in posting_list {
                                wb.set(
                                    format!(
                                        "{}_{}_{}_{}",
                                        SEGMENT_PREFIX, partition, segment_id, &index
                                    )
                                    .into_bytes(),
                                    list,
                                )
                                .unwrap();
                            }
                            wb.set(
                                format!(
                                    "{}_{}_{}_{}",
                                    SEGMENT_PREFIX, partition, segment_id, POSTING_LIST_ALL,
                                )
                                .into_bytes(),
                                entry_offsets,
                            )
                            .unwrap();
                            self.store.flush_batch(wb)?;
                            let segment_file = SegmentFile {
                                id: segment_id,
                                start_ts: start_ts,
                                end_ts: end_ts,
                            };
                            let mut partition_registry: PartitionRegistry;
                            if segment_id == 1 {
                                // This is new segment so, create a new partition registry and save it.
                                let segment_files = vec![segment_file];
                                partition_registry = PartitionRegistry {
                                    last_assigned: 1,
                                    segment_files: segment_files,
                                };
                            } else {
                                let registry_buf = self
                                    .store
                                    .get(format!("{}_{}", PARTITION_PREFIX, partition).as_bytes())?
                                    .unwrap();
                                let mut registry_buf = Deserializer::new(&registry_buf[..]);
                                partition_registry = Deserialize::deserialize(&mut registry_buf)?;
                                // check whether segment persisted in partition registry or not.
                                let mut segment_exist = false;
                                for segment in partition_registry.segment_files.iter_mut() {
                                    if segment.id == segment_file.id {
                                        // If segment exist update the start ts
                                        // and end ts.
                                        segment_exist = true;
                                        segment.start_ts = segment_file.start_ts;
                                        segment.end_ts = segment_file.end_ts;
                                    }
                                }

                                if !segment_exist {
                                    partition_registry.segment_files.push(segment_file);
                                }
                                if partition_registry.last_assigned < segment_id {
                                    assert_eq!(partition_registry.last_assigned, segment_id - 1);
                                    partition_registry.last_assigned = segment_id;
                                }
                            }
                            let mut buf: Vec<u8> = Vec::with_capacity(1024);
                            partition_registry
                                .serialize(&mut Serializer::new(&mut buf))
                                .unwrap();
                            self.store.set(
                                format!("{}_{}", PARTITION_PREFIX, partition).as_bytes(),
                                buf,
                            );

                            // build fst index.
                            indices.sort();
                            let indices = indices.drain(..);
                            let mut index_writer = SetBuilder::memory();
                            for index in indices {
                                index_writer.insert(index).unwrap();
                            }
                            let index_path =
                                partition_path.join(format!("segment_index_{}.fst", segment_id));
                            let mut index_file = File::create(index_path)?;
                            let buf = index_writer.into_inner().unwrap();
                            index_file.write(&buf)?;
                            index_file.flush()?;
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                if e.to_string().contains("invalid segment file") {
                    drop(file);
                    remove_file(segment_path)?;
                    return Ok(());
                }
                return Err(e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::config::config::Config;
    use crate::partition::segment_iterator::SegmentIterator;
    use crate::partition::segment_writer::tests::{get_test_cfg, get_test_store};
    use crate::partition::segment_writer::SegmentWriter;
    use crate::types::types::api::PushLogLine;
    use crate::types::types::LogLine;
    use std::fs::OpenOptions;
    #[test]
    fn test_replayer() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone()).clone();

        // create segment and just write don't close to avoid
        // fst flush.
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
            json_keys: Vec::default(),
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
            json_keys: Vec::default(),
        });
        segment_writer.push(lines).unwrap();
        segment_writer.flush().unwrap();

        // drop the writer so that segment file
        // will be closed.
        drop(segment_writer);
        let mut replayer = Replayer::new(cfg.clone(), store.clone());
        replayer.replay().unwrap();

        // Let's open a segment iterator anc check.
        let partition_path = Path::new(&cfg.dir).join("partition");
        let mut iterator = SegmentIterator::new(
            1,
            partition_path.clone().join("tmppartition"),
            store.clone(),
            None,
            String::from("tmppartition"),
            1,
            5,
            false,
        )
        .unwrap();
        assert_eq!(iterator.entries.len(), 2);
    }

    #[test]
    fn test_replayer_truncate() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone()).clone();

        // create segment and just write don't close to avoid
        // fst flush.
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
            json_keys: Vec::default(),
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
            json_keys: Vec::default(),
        });
        segment_writer.push(lines).unwrap();
        segment_writer.flush().unwrap();

        // drop the writer so that segment file
        // will be closed.
        drop(segment_writer);

        // let's trruncate the file manually and see whether
        let partition_path = Path::new(&cfg.dir).join("partition").join("tmppartition");
        let mut segment_file = OpenOptions::new()
            //  .truncate(true)
            .write(true)
            .read(true)
            .open(&partition_path.join(format!("{}.segment", 1)))
            .unwrap();
        let actual_length = segment_file.metadata().unwrap().len();
        // truncate two bytes.
        segment_file.set_len(100).unwrap();
        segment_file.flush().unwrap();
        drop(segment_file);

        let mut replayer = Replayer::new(cfg.clone(), store.clone());
        replayer.replay().unwrap();

        // Let's open a segment iterator anc check.
        let mut iterator = SegmentIterator::new(
            1,
            partition_path,
            store.clone(),
            None,
            String::from("tmppartition"),
            1,
            5,
            false,
        )
        .unwrap();
        // since two bytes are truncated only one entry will be there.
        assert_eq!(iterator.entries.len(), 1);
    }
}
