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
use failure::Error;
use futures::channel::mpsc;
use futures::channel::oneshot::Sender;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::cmp::{Eq, Ord, PartialEq, PartialOrd};
use tonic::Status;
pub mod api {
    tonic::include_proto!("api"); // The string specified here must match the proto package name
}
use api::QueryResponse as PbQueryResponse;
#[derive(Deserialize, Serialize, Debug)]
pub struct LogLine {
    pub line: String,
    pub ts: u64,
    pub indexes: Vec<String>,
    pub json: bool,
    // always push flattened json keys.
    pub json_keys: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
/// QueryRequest is used for sending query request.
pub struct QueryRequest {
    pub query: String,
    pub start_ts: u64,
    pub end_ts: u64,
    pub count: u64,
    pub offset: u64,
    pub partitions: Vec<String>,
    pub forward: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PartitionRes {
    pub partitions: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResLine {
    pub line: String,
    pub ts: u64,
    pub app: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryResponse {
    pub lines: Vec<ResLine>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PushRequest {
    pub app: String,
    pub lines: Vec<LogLine>,
}

pub const PARTITION_PREFIX: &str = "CHOLA_PARTITION_SHIYALI";

pub const SEGMENT_PREFIX: &str = "CHOLA_SEGMENT";

pub const SEGEMENT_JSON_KEY_PREFIX: &str = "CHOLA_JSON_KEY";

// use to get the all the offset of the segment files.
// IYANAN is one of the thamizha sangam name. which
// means creator.
pub const POSTING_LIST_ALL: &str = "!CHOLA!IYANAN";

#[derive(Deserialize, Serialize)]
pub struct PartitionRegistry {
    pub last_assigned: u64,
    pub segment_files: Vec<SegmentFile>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SegmentFile {
    pub start_ts: u64,
    pub end_ts: u64,
    pub id: u64,
}

impl Ord for SegmentFile {
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_ts.cmp(&other.start_ts)
    }
}

impl Eq for SegmentFile {}

impl PartialOrd for SegmentFile {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SegmentFile {
    fn eq(&self, other: &Self) -> bool {
        self.start_ts == other.start_ts
    }
}

pub enum IngesterRequest {
    Push(IngesterPush),
    Flush(IngesterFlushHintReq),
    RegisterTailer(TailerRequest),
}

pub struct TailerRequest {
    pub partitions: Vec<String>,
    pub sender: mpsc::Sender<Result<PbQueryResponse, Status>>,
}

#[derive(Debug)]
pub struct IngesterPush {
    pub push_request: api::PushRequest,
    pub complete_signal: Sender<Result<(), Error>>,
}

#[derive(Debug)]
pub struct IngesterFlushHintReq {
    pub app: String,
    pub start_ts: u64,
    pub end_ts: u64,
    pub complete_signal: Sender<Result<(), Error>>,
}
