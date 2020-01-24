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
use std::thread;
use std::time::Duration;
/// The CronJob trait allows to execute the cron job.
pub trait CronJob {
    fn execute(&self);
}

/// The CronScheduler allows to execute job for given interval of time.
pub struct CronScheduler<T> {
    jobs: Vec<T>,
    interval: Duration,
}

impl<'a, T: CronJob + Send + 'static> CronScheduler<T> {
    /// new gives CrobScheduler by taking jobs and duration as an input.
    pub fn new(jobs: Vec<T>, interval: Duration) -> CronScheduler<T> {
        CronScheduler {
            jobs: jobs,
            interval: interval,
        }
    }

    /// start will execute the given job for given interval.
    pub fn start(self) {
        thread::spawn(move || loop {
            for job in &self.jobs {
                job.execute();
            }
            thread::sleep(self.interval.clone());
        });
    }
}
