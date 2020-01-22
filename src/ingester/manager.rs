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
use crate::ingester::ingester::Ingester;
use crate::store::rocks_store::RocksStore;
use crate::types::types::api::QueryResponse;
use crate::types::types::{IngesterFlushHintReq, IngesterPush, IngesterRequest, TailerRequest};
use failure;
use futures::channel::mpsc;
use futures::channel::mpsc::Sender;
use futures::executor::block_on;
use futures::sink::SinkExt;
use num_cpus;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::thread;
use tonic::Status;
/// Manager is responsible for managing multiple ingester and route the loglines to the correct
/// ingester.
#[derive(Clone)]
pub struct Manager {
    pub transport: Vec<Sender<IngesterRequest>>,
    pub no_of_shard: usize,
}

impl Manager {
    pub fn new(cfg: Config, store: RocksStore) -> Manager {
        let mut manager = Manager {
            transport: Vec::default(),
            no_of_shard: num_cpus::get(),
        };
        // Spin one ingester for each cpu.
        for _ in 0..manager.no_of_shard {
            let (sender, receiver) = mpsc::channel(1000);
            let mut ingester = Ingester::new(receiver, cfg.clone(), store.clone());
            manager.transport.push(sender);
            // Start the ingester in a new thread.
            thread::spawn(move || {
                ingester.start();
            });
        }
        manager
    }

    pub async fn ingest(&mut self, req: IngesterPush) -> Result<(), failure::Error> {
        // Send the request to the right shard.
        let shard = self.get_ingester_shard(&req.push_request.source);
        if shard >= self.no_of_shard {
            panic!(shard);
        }
        let sender = self.transport.get_mut(shard).unwrap();
        sender.send(IngesterRequest::Push(req)).await?;
        Ok(())
    }

    fn get_ingester_shard(&mut self, source: &String) -> usize {
        // Calculate the shard to send the request to the right partition.
        // IDEAS: check io_uring how it scales well. Otherwise, instead of hashing based routing use
        // some logic to find the right shard. Example worker stealing.
        let mut s = DefaultHasher::new();
        source.hash(&mut s);
        let hash_value = s.finish();
        // Get the right ingester for the given source.
        (hash_value as usize) % self.no_of_shard
    }

    pub fn register_tailer(
        &mut self,
        sources: Vec<String>,
        tailer_sender: Sender<Result<QueryResponse, Status>>,
    ) -> Result<(), failure::Error> {
        // We'll batch the partitions based on the shard.
        let mut buckets: HashMap<usize, TailerRequest> = HashMap::new();
        for partition in sources {
            let shard = self.get_ingester_shard(&partition);
            if let Some(req) = buckets.get_mut(&shard) {
                req.partitions.push(partition);
                continue;
            }
            buckets.insert(
                shard,
                TailerRequest {
                    partitions: vec![partition],
                    sender: tailer_sender.clone(),
                },
            );
        }

        // Send the batched request to the right ingester.
        for (shard, batched_req) in buckets {
            let sender = self.transport.get_mut(shard).unwrap();

            block_on(async {
                sender
                    .send(IngesterRequest::RegisterTailer(batched_req))
                    .await
            })?;
        }

        Ok(())
    }

    pub fn send_flush_hint(&mut self, req: IngesterFlushHintReq) -> Result<(), failure::Error> {
        // Get the right sender for the given request.
        let shard = self.get_ingester_shard(&req.app);
        let sender = self.transport.get_mut(shard).unwrap();

        block_on(async { sender.send(IngesterRequest::Flush(req)).await })?;
        Ok(())
    }
}
