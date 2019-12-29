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
pub mod decode;
pub mod util;
use byteorder::LittleEndian;
use byteorder::ReadBytesExt;
use std::io::Cursor;
/// decode_u64 is used to decode the buf.
pub fn decode_u64(buf: &[u8]) -> u64 {
    let mut reader = Cursor::new(buf);
    // let it panic, If any invalid data.
    reader.read_u64::<LittleEndian>().unwrap()
}
