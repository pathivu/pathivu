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
// NEED to look into it. why it is slow.
// use crate::config::config::Config;
// use crate::store::batch::Batch as StoreBatch;
// use crate::store::store::Store;
// use failure::{bail, Error};
// use log::{debug, info, warn};
// use sled::{Batch, ConfigBuilder, Db};

// /// concatenate_merge will merge the previous value with the give value.
// // copied from sled documentation.
// fn concatenate_merge(
//     _key: &[u8],              // the key being merged
//     old_value: Option<&[u8]>, // the previous value, if one existed
//     merged_bytes: &[u8],      // the new bytes being merged in
// ) -> Option<Vec<u8>> {
//     // set the new value, return None to delete
//     let mut ret = old_value.map(|ov| ov.to_vec()).unwrap_or_else(|| vec![]);

//     ret.extend_from_slice(merged_bytes);

//     Some(ret)
// }

// pub struct SledStore {
//     db: Db,
// }

// impl SledStore {
//     pub fn new(cfg: Config) -> Result<SledStore, Error> {
//         let config = ConfigBuilder::default()
//             .path(cfg.dir)
//             .cache_capacity(10_000_000_000)
//             .flush_every_ms(Some(1000))
//             .snapshot_after_ops(100_000)
//             .build();
//         let db = Db::start(config)?;
//         db.set_merge_operator(concatenate_merge);
//         Ok(SledStore { db: db })
//     }
// }

// impl Store for SledStore {
//     fn merge(&mut self, key: &[u8], value: Vec<u8>) {
//         self.db.merge(key, value);
//     }

//     fn flush(&mut self) -> Result<usize, Error> {
//         match self.db.flush() {
//             Ok(size) => Ok(size),
//             Err(e) => bail!("{}", e),
//         }
//     }

//     fn set(&mut self, key: &[u8], value: Vec<u8>) {
//         self.db.insert(key, value);
//     }

//     fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, failure::Error> {
//         let res = self.db.get(key)?;
//         info!("{:?}", res);
//         match res {
//             Some(value) => Ok(Some(value.to_vec())),
//             None => Ok(None),
//         }
//     }
//     fn flush_batch(&self, wb: Batch) -> Result<(), failure::Error> {
//         panic!("yet to implement batch for sled");
//         Ok(())
//     }
// }

// impl Clone for SledStore {
//     fn clone(&self) -> SledStore {
//         SledStore {
//             db: self.db.clone(),
//         }
//     }
// }
