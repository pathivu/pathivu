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
use crate::partition::segment_iterator::{decode_entry, Entry};
use crate::util::decode_u64;
use failure;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::rc::Rc;
///FileIterator is used to iterate over all the log lines till the given offset.
pub struct FileIterator {
    entries: Vec<Rc<Entry>>,
    current_index: usize,
}

impl FileIterator {
    pub fn new(
        id: u64,
        partition: String,
        dir: String,
        end_offset: u64,
        start_ts: u64,
        end_ts: u64,
    ) -> Result<FileIterator, failure::Error> {
        let path = Path::new(&dir)
            .join("partition")
            .join(&partition)
            .join(format!("{}.segment", id));
        // allocate buffer.
        let mut buffer = vec![0; end_offset as usize];
        let mut file = File::open(path)?;
        // read the exact bytes.
        file.read_exact(&mut buffer[..])?;
        assert_eq!(buffer.len(), end_offset as usize);
        let mut entries = Vec::new();
        let mut read_index = 14 as usize;
        loop {
            if read_index >= end_offset as usize {
                break;
            }
            let entry_len = decode_u64(&buffer[read_index..read_index + 8]) as usize;
            read_index = read_index + 8;
            let entry = decode_entry(&buffer[read_index..read_index + entry_len]);
            if start_ts <= entry.ts && entry.ts <= end_ts {
                entries.push(Rc::new(entry));
            }
            read_index = read_index + entry_len;
        }
        Ok(FileIterator {
            entries: entries,
            current_index: 0,
        })
    }

    pub fn entry(&self) -> Option<Rc<Entry>> {
        let entry = self.entries.get(self.current_index);
        match entry {
            Some(ent) => Some(ent.clone()),
            None => None,
        }
    }

    /// next will advance the iterator. throws error if we reach end.
    pub fn next(&mut self) -> Option<()> {
        if self.current_index >= self.entries.len() - 1 {
            // just incrementing one so that entry will give none.
            self.current_index = self.current_index + 1;
            return None;
        }
        self.current_index = self.current_index + 1;
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use crate::partition::segment_file_iterator::FileIterator;
    use crate::partition::segment_writer::tests::{get_test_cfg, get_test_store};
    use crate::partition::segment_writer::SegmentWriter;
    use crate::types::types::LogLine;
    #[test]
    fn test_file_iterator() {
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
        lines.push(LogLine {
            line: String::from("liala transfered money to raja"),
            indexes: vec![
                "liala".to_string(),
                "transfered".to_string(),
                "money".to_string(),
                "raja".to_string(),
            ],
            ts: 2,
            json: false,
            json_keys: Vec::default(),
        });
        lines.push(LogLine {
            line: String::from("roja transfered money to navin"),
            indexes: vec![
                "roja".to_string(),
                "transfered".to_string(),
                "money".to_string(),
                "navin".to_string(),
            ],
            ts: 4,
            json: false,
            json_keys: Vec::default(),
        });
        segment_writer.push(lines).unwrap();
        segment_writer.flush().unwrap();
        let mut iterator = FileIterator::new(
            1,
            String::from("tmppartition"),
            cfg.dir,
            segment_writer.size(),
            1,
            5,
        )
        .unwrap();
        let ent = iterator.entry().unwrap();
        assert_eq!(ent.ts, 2);
        iterator.next().unwrap();
        let ent = iterator.entry().unwrap();
        assert_eq!(ent.ts, 4);
    }
}
