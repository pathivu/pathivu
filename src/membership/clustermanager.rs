use crate::config::config::Config;
use crate::store::store::Store;
use crate::types::types::{ASK_PUSH_ADDR_COMMAND, PATHIVU_CLUSTER_ID_KEY};
use artillery_core::epidemic::prelude::*;
use failure;
use std::collections::HashMap;
use std::convert::TryInto;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use uuid::Uuid;

/// ClusterManager is responsible for handling cluster membership and sharding the source with
/// the right destination.
#[derive(Default)]
pub struct ClusterManager {
    /// id of the this pathivu node.
    id: uuid::Uuid,
    /// peers info of all the nodes in the pathivu cluster.
    peers: Arc<Mutex<HashMap<Uuid, ArtilleryMember>>>,
    /// push_addrs holds the information about all the pathivu push address.
    push_addrs: Arc<Mutex<HashMap<Uuid, String>>>,
}

impl ClusterManager {
    /// new will create cluster manager.
    pub fn new<S: Store>(mut store: S) -> Result<ClusterManager, failure::Error> {
        let val = store.get(PATHIVU_CLUSTER_ID_KEY.as_bytes())?;
        // Get the server id from the store.
        let id = val.map_or_else(
            || {
                let id = Uuid::new_v4();
                store.set(PATHIVU_CLUSTER_ID_KEY.as_bytes(), id.as_bytes().to_vec());
                id
            },
            |id| {
                let u: [u8; 16] = id.as_slice().try_into().unwrap();
                Uuid::from_bytes(u)
            },
        );
        Ok(ClusterManager {
            id: id,
            ..Default::default()
        })
    }

    /// start will start the artillery membership and maintains the cluster state.
    pub fn start(&mut self, cfg: Config) {
        let cluster_config = ClusterConfig {
            cluster_key: format!("pathivu").as_bytes().to_vec(),
            listen_addr: (&cfg.peer_addr as &str)
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap(),
            ..Default::default()
        };

        let cluster = Cluster::new_cluster(self.id, cluster_config).unwrap();
        println!("gossip peer is listening on {:?}", cfg.peer_addr);
        // Add all the peer to the cluster to discovers themself.
        cfg.peers.iter().for_each(|peer| {
            println!("adding seed peer to the cluster {:?}", peer);
            cluster.add_seed_node(FromStr::from_str(peer).unwrap());
        });

        let peers = self.peers.clone();
        let push_addrs = self.push_addrs.clone();
        thread::spawn(move || {
            for (members, event) in cluster.events.iter() {
                println!("members {:?} events {:?}", members, event);
                match event {
                    ArtilleryMemberEvent::MemberJoined(member) => {
                        // Ask the push address.
                        cluster.send_payload(member.host_key(), ASK_PUSH_ADDR_COMMAND);
                        // Add the joined peer to the local state.
                        let mut safe_peers = peers.lock().unwrap();
                        safe_peers.insert(member.host_key(), member);
                    }
                    ArtilleryMemberEvent::MemberPayload(member, payload) => {
                        if payload == ASK_PUSH_ADDR_COMMAND {
                            cluster.send_payload(member.host_key(), cfg.http_addr.clone());
                            continue;
                        }

                        let mut safe_push_addrs = push_addrs.lock().unwrap();
                        safe_push_addrs.insert(member.host_key(), payload);
                    }
                    _ => {
                        // TODO: handle member dead and went up.
                    }
                }
            }
        });
    }
}
