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
#![feature(async_closure)]
#![feature(type_ascription)]
#![feature(result_map_or_else)]
mod config;
mod cronscheduler;
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
use std::time::Duration;
mod retention;
mod types;
mod util;
use clap::{App, Arg};
use simplelog::*;
use std::fs::create_dir_all;
use std::path::Path;
fn main() {
    // CombinedLogger::init(vec![
    //     TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed).unwrap(),
    //     TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap(),
    //     TermLogger::new(LevelFilter::Debug, Config::default(), TerminalMode::Mixed).unwrap(),
    // ])
    // .unwrap();
    let matches = App::new("pathivu")
        .version("v0.1")
        .about("Pathivu: Logs you can search")
        .arg(
            Arg::with_name("dir")
                .short("d")
                .long("dir")
                .value_name("DIR")
                .help("directory to store logs")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("retension")
                .short("r")
                .long("ret")
                .help("log retention in days")
                .takes_value(true),
        )
        .get_matches();

    let root_dir = Path::new(matches.value_of("dir").unwrap_or(".")).join("pathivu_data");
    create_dir_all(&root_dir).unwrap();
    let days_to_keep = matches
        .value_of("retension")
        .unwrap_or("10")
        .parse::<u64>()
        .expect("argument retension should be in number");
    let cfg = config::config::Config {
        dir: root_dir,
        max_segment_size: 100 << 10,
        max_index_size: 100 << 10,
        max_batch_size: 20,
        retention_period: days_to_keep * 26 * 60 * 60,
    };
    let store = store::rocks_store::RocksStore::new(cfg.clone()).unwrap();

    // Run all cron jobs.
    let mut jobs: Vec<Box<dyn cronscheduler::cron_scheduler::CronJob + Send>> = Vec::new();
    // Add telementry job.
    jobs.push(Box::new(telementry::telementry::TelementryJob {}));
    // Add Rentention manager.
    jobs.push(Box::new(retention::retention::RententionManager::new(
        cfg.clone(),
        store.clone(),
    )));

    cronscheduler::cron_scheduler::CronScheduler::new(jobs, Duration::from_secs(3600)).start();

    // TODO: refactor server to loose couple ingester and query executor from
    // server.
    server::server::Server::start(cfg, store).unwrap();
}
