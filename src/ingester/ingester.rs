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
use crate::partition::segment_writer::SegmentWriter;
use crate::store::store::Store;
use crate::types::types::*;
use futures::channel::mpsc::{Receiver, Sender};
use futures::executor::block_on;
use futures::sink::SinkExt;
use futures::stream::StreamExt;
use log::{debug, info, warn};
use retain_mut::RetainMut;
use rmp_serde::Deserializer;
use serde::Deserialize;
use std::collections::HashMap;
use tonic::Status;

pub struct Ingester<S: Store> {
    receiver: Receiver<IngesterRequest>,
    id: u8,
    segment_writers: HashMap<String, SegmentWriter<S>>,
    cfg: Config,
    store: S,
    tailers: HashMap<String, Vec<Sender<Result<api::QueryResponse, Status>>>>,
}

impl<S: Store + Clone> Ingester<S> {
    pub fn new(receiver: Receiver<IngesterRequest>, cfg: Config, store: S) -> Ingester<S> {
        Ingester {
            receiver: receiver,
            id: 0,
            segment_writers: HashMap::new(),
            cfg: cfg,
            store: store,
            tailers: HashMap::default(),
        }
    }

    pub fn start(&mut self) {
        info!("ingester {} started", self.id);
        loop {
            let ingester_request = block_on(async { self.receiver.next().await });
            info!("received yo");
            if !ingester_request.is_some() {
                continue;
            }
            let ingester_request = ingester_request.unwrap();
            match ingester_request {
                IngesterRequest::Push(req) => {
                    // first handle tailers.
                    self.handle_tailers(&req);

                    // now persist in the segment writer.
                    let result = self.push(req.push_request);
                    info!(" result {:?}", result);
                    match req.complete_signal.send(result) {
                        Err(e) => {
                            warn!(
                                "unable to complete the signal for the ingester {}: {:?}",
                                self.id, e
                            );
                        }
                        _ => {}
                    }
                }
                IngesterRequest::Flush(hint) => {
                    let result = self.flush_if_necessary(hint.app, hint.start_ts, hint.end_ts);
                    match hint.complete_signal.send(result) {
                        Err(e) => warn!(
                            "unable to send complete signal for ingester necessary flush {:?}",
                            e
                        ),

                        _ => debug!("ingester necessary flush signal sent successfully"),
                    }
                }
                IngesterRequest::RegisterTailer(req) => {
                    self.register_tailer(req);
                }
            }
        }
    }

    fn flush_if_necessary(
        &mut self,
        partition: String,
        start_ts: u64,
        end_ts: u64,
    ) -> Result<(), failure::Error> {
        if let Some(writer) = self.segment_writers.get_mut(&partition) {
            let (segment_start_ts, segment_end_ts) = writer.segment_ts();
            if (segment_start_ts >= start_ts && segment_start_ts <= end_ts)
                || (segment_end_ts >= start_ts && segment_end_ts <= start_ts)
                || (start_ts == 0 && end_ts == 0)
            {
                let segment_writer = self.segment_writers.remove(&partition).unwrap();
                segment_writer.close()?;
                debug!("flushing writer {} for hint", partition);
            }
        }
        Ok(())
    }

    fn push(&mut self, req: api::PushRequest) -> Result<(), failure::Error> {
        if req.lines.len() == 0 {
            return Ok(());
        }
        debug!(
            "ingesting partition {}, with {} lines",
            req.source,
            req.lines.len()
        );
        let ref mut segment_writer: SegmentWriter<S>;
        if let Some(writer) = self.segment_writers.get_mut(&req.source) {
            segment_writer = writer;
            info!("writer is thre");
        } else {
            info!("writer not there");
            let writer = self.create_segment_writer(&req.source, req.lines[0].ts)?;
            info!("inserting yo");
            self.segment_writers.insert(req.source.clone(), writer);
            segment_writer = self.segment_writers.get_mut(&req.source).unwrap();
        }
        segment_writer.push(req.lines)?;
        if self.cfg.max_segment_size <= segment_writer.size()
        //|| self.cfg.max_index_size <= segment_writer.index_size()
        {
            let segment_writer = self.segment_writers.remove(&req.source).unwrap();
            segment_writer.close()?;
        }
        info!("segment writers {:}", &self.segment_writers.len());
        Ok(())
    }
    /// create_segment_writer creates segment writer for the given partition.
    fn create_segment_writer(
        &self,
        partition: &String,
        start_ts: u64,
    ) -> Result<SegmentWriter<S>, failure::Error> {
        let segment_id: u64;
        let partition_registry = self
            .store
            .get(format!("{}_{}", PARTITION_PREFIX, partition).as_bytes())?;
        match partition_registry {
            Some(registry) => {
                let mut buf = Deserializer::new(&registry[..]);
                let registry: PartitionRegistry = Deserialize::deserialize(&mut buf)?;
                segment_id = registry.last_assigned + 1;
            }
            None => segment_id = 1,
        }
        SegmentWriter::new(
            self.cfg.clone(),
            partition.clone(),
            segment_id,
            self.store.clone(),
            start_ts,
        )
    }

    fn register_tailer(&mut self, mut req: TailerRequest) {
        // push_tailer will push the tailer in the ingester tailer mapping.
        let mut push_tailer =
            |key: String, tailer: Sender<Result<api::QueryResponse, Status>>| match self
                .tailers
                .get_mut(&key)
            {
                Some(tailers) => tailers.push(tailer),
                None => {
                    self.tailers.insert(key, vec![tailer]);
                }
            };

        if req.partitions.len() == 0 {
            push_tailer(String::from("*"), req.sender);
            return;
        }

        for partition in req.partitions.drain(..) {
            push_tailer(partition, req.sender.clone());
        }
    }

    fn handle_tailers(&mut self, req: &IngesterPush) {
        // send_logs send logs to the respective listener stream.
        let send_logs = |tailers: Option<&mut Vec<Sender<Result<api::QueryResponse, Status>>>>| {
            if let Some(tailers) = tailers {
                tailers.retain_mut(|tailer| {
                    let mut lines = Vec::new();
                    for log_line in req.push_request.lines.iter() {
                        lines.push(api::LogLine {
                            app: req.push_request.source.clone(),
                            inner: String::from_utf8(log_line.raw_data.clone()).unwrap(),
                            ts: log_line.ts,
                            structured: log_line.structured,
                        });
                    }
                    match block_on(async {
                        tailer
                            .send(Ok(api::QueryResponse {
                                lines: lines,
                                json: String::from(""),
                            }))
                            .await
                    }) {
                        Ok(_) => true,
                        Err(_) => {
                            println!("removing tailer");
                            return false;
                        }
                    }
                });
            }
        };

        // First send for wildcards.
        let tailers = self.tailers.get_mut("*");
        send_logs(tailers);

        // Then send it to the app specific listeners.
        let tailers = self.tailers.get_mut(&req.push_request.source);
        send_logs(tailers);
    }
}
