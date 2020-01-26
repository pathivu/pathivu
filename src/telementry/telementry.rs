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
use crate::cronscheduler::cron_scheduler::CronJob;
use isahc::prelude::*;
use log::{info, warn};
use serde::Serialize;
use std::thread;
use std::time::{Duration, SystemTime};
const API_KEY: &str = "c4854d00f1ac7703a0648ccdad037b9c";

#[derive(Serialize)]
struct Event<'a> {
    time: u64,
    user_id: &'a str,
    platform: &'a str,
    event_type: &'a str,
}

#[derive(Serialize)]
struct TelementryRequest<'a> {
    api_key: &'a str,
    events: Vec<Event<'a>>,
}

/// TelementryJob send telementry to the amplitude.
pub struct TelementryJob {}

impl CronJob for TelementryJob {
    fn execute(&mut self) {
        let username = whoami::username();
        let hostname = whoami::hostname();
        let platform = whoami::platform().to_string();
        let user_id = format!("{}_{}_{}", username, hostname, platform);
        let time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let time = time.as_secs();
        let event_type = "alive_tick";
        let event = Event {
            time: time,
            user_id: &user_id,
            platform: &platform,
            event_type: &event_type,
        };
        let body = TelementryRequest {
            api_key: API_KEY,
            events: vec![event],
        };

        let body = serde_json::to_string(&body).unwrap();
        Request::post("https://api.amplitude.com/2/httpapi")
            .header("Content-Type", "application/json")
            .timeout(Duration::from_secs(5))
            .body(body)
            .map_or_else(
                |e| {
                    warn!("unable to create telementry request with body {}", e);
                },
                |req| {
                    req.send().map_or_else(
                        |e| {
                            warn!("unable to send telementry request {}", e);
                        },
                        |res| {
                            if !res.status().is_success() {
                                warn!("failed on sending telementry request {:?}", res.body())
                            }
                        },
                    );
                },
            );
    }
}
