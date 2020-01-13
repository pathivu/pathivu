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
#![feature(or_patterns)]
#![feature(async_closure)]
#![feature(type_ascription)]
#![feature(result_map_or_else)]
mod config;
mod ingester;
mod iterator;
mod json_parser;
mod parser;
mod partition;
mod queryexecutor;
mod replayer;
mod server;
mod store;
mod telementry;
mod types;
mod util;
use simplelog::*;
fn main() {
    // CombinedLogger::init(vec![
    //     TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed).unwrap(),
    //     TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap(),
    //     TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed).unwrap(),
    // ])
    // .unwrap();

    // Send telementry for analytics.
    telementry::telementry::send_telementry();
    // TODO: refactor server to loose couple ingester and query executor from
    // server.
    server::server::Server::start();
}
