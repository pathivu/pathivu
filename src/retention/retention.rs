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
use crate::cronscheduler::cron_scheduler::CronJob;
use crate::server::server::get_partitions;
use crate::store::store::Store;
use crate::types::types::{PartitionRegistry, PARTITION_PREFIX};
use log::warn;
use rmp_serde::decode::Error as RmpError;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use std::fs::remove_file;
use std::path::Path;
use std::time::SystemTime;
pub struct RententionManager<S> {
    store: S,
    cfg: Config,
}

impl<S: Store> RententionManager<S> {
    pub fn new(cfg: Config, store: S) -> RententionManager<S> {
        RententionManager {
            store: store,
            cfg: cfg,
        }
    }
}

impl<S: Store> CronJob for RententionManager<S> {
    fn execute(&mut self) {
        // TOOD: do have some kind of locking mechanishm before
        // deleting the partition files. use a global readn write lock to get
        // the lock.
        let partitions = get_partitions(&self.cfg.dir);

        if partitions.is_err() {
            warn!(
                "error while getting partitions in retention manager {}",
                partitions.unwrap_err()
            );
            return;
        }

        let partitions = partitions.unwrap();

        for partition in partitions {
            let buf = self
                .store
                .get(format!("{}_{}", PARTITION_PREFIX, partition).as_bytes());

            if buf.is_err() {
                warn!(
                    "error while retriving partititon registry buf in retention manager {}",
                    buf.unwrap_err()
                );
                continue;
            }
            let buf = buf.unwrap();

            if buf.is_none() {
                // Segement files are not flused. Go for the other partition silently.
                continue;
            }
            let buf = buf.unwrap();
            let mut buf = Deserializer::new(&buf[..]);
            let registry: Result<PartitionRegistry, RmpError> = Deserialize::deserialize(&mut buf);

            if registry.is_err() {
                warn!(
                    "error while deserializing partition registry in retension manager {:?}",
                    registry.unwrap_err()
                );
                continue;
            }
            let mut registry = registry.unwrap();

            let mut segements_needs_to_be_deleted = Vec::new();
            let mut filtered_segment_file = Vec::new();
            for segment_file in registry.segment_files {
                // Filter segment file that needs to be filtered.
                let now = SystemTime::now().duration_since(std::time::UNIX_EPOCH);
                if now.is_err() {
                    warn!(
                        "error while getting unix epoc {:?} in retension manager",
                        now.unwrap()
                    );
                    continue;
                }
                let now = now.unwrap();
                let diff = now.as_secs() - segment_file.end_ts;
                if diff > self.cfg.retention_period {
                    // Delete this segement file.
                    segements_needs_to_be_deleted.push(segment_file.clone());
                    continue;
                }
                filtered_segment_file.push(segment_file.clone());
            }

            // Rebuild the partition registry and delete those segment files.
            registry.segment_files = filtered_segment_file;

            let mut buf = Vec::with_capacity(1024);
            registry.serialize(&mut Serializer::new(&mut buf)).unwrap();
            self.store.set(
                format!("{}_{}", PARTITION_PREFIX, partition).as_bytes(),
                buf,
            );

            // Delete the segment file respectivelty.
            let partition_path = Path::new(&self.cfg.dir).join("partition").join(&partition);

            for segment_file in segements_needs_to_be_deleted {
                // Delete index file.
                if let Err(e) = remove_file(
                    &partition_path.join(format!("segment_index_{}.fst", segment_file.id)),
                ) {
                    warn!("unable to delete the index file {} for the partition {} in rentension manager {}",segment_file.id, &partition,e);
                }
                // Delete segment file.
                if let Err(e) =
                    remove_file(&partition_path.join(format!("{}.segment", segment_file.id)))
                {
                    warn!("unable to delete the segment file {} for the partition {} in rentension manager {}",segment_file.id, &partition,e);
                }
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::partition::partition_iterator::tests::create_segment;
    use crate::partition::segment_writer::tests::{get_test_cfg, get_test_store};
    use crate::types::types::{PartitionRegistry, PARTITION_PREFIX};
    use rmp_serde::Deserializer;
    use serde::Deserialize;
    use std::time::SystemTime;

    #[test]
    fn test_retension_manager() {
        let cfg = get_test_cfg();
        let store = get_test_store(cfg.clone());
        let partition_name = String::from("kube-server");

        let previous_time = SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - cfg.retention_period
            - 1000;
        create_segment(
            1,
            partition_name.clone(),
            cfg.clone(),
            previous_time,
            store.clone(),
        );

        let mut retention_manager = RententionManager {
            cfg: cfg.clone(),
            store: store.clone(),
        };

        retention_manager.execute();
        let buf = store
            .get(format!("{}_{}", PARTITION_PREFIX, partition_name).as_bytes())
            .unwrap()
            .unwrap();
        let mut buf = Deserializer::new(&buf[..]);
        let registry: PartitionRegistry = Deserialize::deserialize(&mut buf).unwrap();
        assert_eq!(0, registry.segment_files.len());
        let path = std::path::Path::new(&cfg.dir)
            .join("partition")
            .join(&partition_name);
        let dir = std::fs::read_dir(&path).unwrap();
        let mut len = 0;
        for _ in dir {
            len = len + 1;
        }
        assert_eq!(len, 0);
    }
}
