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
use failure::bail;
use failure::Error;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// Iterator helps to iterate over entry in segment files.
pub struct Iterator {
    fs: File,
    size: u64,
    valid_offset: u64,
}

impl Iterator {
    /// new will give the iterator.
    pub fn new(mut fs: File) -> Result<Iterator, Error> {
        let info = fs.metadata()?;
        if info.len() < 14 {
            // layout itself not generated so, let's delete the file.
            return Err(bail!("invalid segment file"));
        }
        fs.seek(SeekFrom::Start(14))?;
        Ok(Iterator {
            fs: fs,
            size: info.len() as u64,
            valid_offset: 14,
        })
    }
    /// iterate will iterate entry of the segment file one by one.
    pub fn iterate(&mut self) -> Result<Option<Entry>, Error> {
        if self.valid_offset + 8 > self.size {
            // we reached end of file. So, we'll truncate till valid offset.
            return Ok(None);
        }
        // check whether we have entry from the valid offset or not.
        let mut buf: [u8; 8] = [0; 8];
        self.fs.read(&mut buf)?;
        let mut current_offset = self.valid_offset + 8;
        let entry_len = decode_u64(&buf);

        if current_offset + entry_len > self.size {
            // we only persisted entry length not the entry so skipping here.
            return Ok(None);
        }
        // read the entry.
        let mut entry_buf: Vec<u8> = vec![0; entry_len as usize];
        self.fs.read(&mut entry_buf[..])?;
        current_offset = current_offset + entry_len;
        let entry = decode_entry(&entry_buf);
        // update the current valid offset.
        self.valid_offset = current_offset;
        Ok(Some(entry))
    }

    /// valid_offset return the valid offset of the segment file.
    /// It'll be used to truncate the segment file.
    pub fn valid_offset(&self) -> u64 {
        self.valid_offset
    }
}
