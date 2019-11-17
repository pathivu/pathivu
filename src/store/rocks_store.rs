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
use crate::store::batch::Batch as StoreBatch;
use crate::store::store::Store;
use failure;
use failure::bail;
use log::{debug, info, warn};
use rocksdb::{Writable, WriteBatch, DB};
use std::sync::Arc;
pub struct RocksStore {
    db: Arc<DB>,
}

impl RocksStore {
    pub fn new(cfg: Config) -> Result<RocksStore, failure::Error> {
        let db = DB::open_default(&cfg.dir).unwrap();
        Ok(RocksStore { db: Arc::new(db) })
    }
}

impl Store for RocksStore {
    fn merge(&mut self, key: &[u8], value: Vec<u8>) {
        self.db.merge(key, &value);
    }

    fn flush(&mut self) -> Result<usize, failure::Error> {
        match self.db.flush(false) {
            Ok(size) => Ok(0),
            Err(e) => bail!("{}", e),
        }
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) {
        self.db.put(key, &value);
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, failure::Error> {
        match self.db.get(key) {
            Ok(res) => match res {
                Some(val) => Ok(Some(val.to_vec())),
                None => Ok(None),
            },
            Err(e) => bail!("{}", e),
        }
    }
    fn flush_batch(&self, wb: StoreBatch) -> Result<(), failure::Error> {
        let mut inner = wb.inner();
        let rwb = WriteBatch::with_capacity(inner.len());
        for (key, value) in inner.drain(0..inner.len()) {
            rwb.put(&key[..], &value[..]).unwrap();
        }
        self.db.write(&rwb).unwrap();
        Ok(())
    }
}

impl Clone for RocksStore {
    fn clone(&self) -> RocksStore {
        RocksStore {
            db: self.db.clone(),
        }
    }
}
