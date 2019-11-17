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
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use failure;
use std::io::Cursor;
/// decode_posting_list is used to decode the given buf to the slices of u64.
/// This can be further optimized for the future by groupvarint compressing.
pub fn decode_posting_list(list: &[u8]) -> Result<Vec<u64>, failure::Error> {
    // posting list is byte of u64.
    assert_eq!(list.len() % 8, 0);
    let mut reader = Cursor::new(list);
    let mut posting_list = Vec::new();
    loop {
        match reader.read_u64::<LittleEndian>() {
            Ok(val) => {
                posting_list.push(val);
            }
            _ => {
                // Should ideally throw an error.
                break;
            }
        }
    }
    Ok(posting_list)
}
