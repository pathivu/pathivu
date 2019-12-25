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
use crate::queryexecutor::executor::QueryExecutor;
use crate::replayer::replayer::Replayer;
use crate::store::batch::Batch;
use crate::store::rocks_store;
use crate::store::store::Store;
use crate::types::types::*;

use api::server::PathivuServer;
use failure::bail;
use futures::channel::mpsc;
use futures::channel::mpsc::Sender;
use futures::channel::oneshot;
use futures::executor::block_on;
use futures::sink::SinkExt;
use gotham;
use gotham::error::Result as GothamResult;
use gotham::handler::{Handler, HandlerFuture, IntoHandlerError, IntoResponse, NewHandler};
use gotham::helpers::http::response::create_empty_response;
use gotham::helpers::http::response::create_response;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};
use hyper::{Body, HeaderMap, Method, Response, StatusCode, Uri, Version};
use log::{debug, info, warn};
use mime;
use oldfuture;
use oldfuture::future::Future;
use oldfuture::stream::Stream;
use rocksdb::WriteBatch;
use std::fs;
use std::fs::create_dir_all;
use std::marker::PhantomData;
use std::panic::RefUnwindSafe;
use std::path::Path;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;
use tokio::runtime::Runtime;
use tonic;
use tonic::{transport::Server as TonicServer, Code, Request, Response as TonicResponse, Status};
struct PathivuGrpcServer {
    ingester_transport: Sender<IngesterRequest>,
    query_executor: QueryExecutor<rocks_store::RocksStore>,
    partition_path: String,
}

#[tonic::async_trait]
impl api::server::Pathivu for PathivuGrpcServer {
    type TailStream = mpsc::Receiver<Result<api::QueryResponse, Status>>;

    async fn tail(
        &self,
        req: Request<api::QueryRequest>,
    ) -> Result<TonicResponse<Self::TailStream>, Status> {
        let (mut tx, rx) = mpsc::channel(100);
        let req = req.into_inner();
        let tailer_req = TailerRequest {
            partitions: req.partitions,
            sender: tx,
        };
        let mut ingester_transport = self.ingester_transport.clone();
        block_on(async {
            ingester_transport
                .send(IngesterRequest::RegisterTailer(tailer_req))
                .await
        });
        Ok(TonicResponse::new(rx))
    }

    async fn query(
        &self,
        req: Request<api::QueryRequest>,
    ) -> Result<TonicResponse<api::QueryResponse>, Status> {
        let req = req.into_inner();
        // convert into executor request.
        let executor_req = QueryRequest {
            partitions: req.partitions,
            start_ts: req.start_ts,
            end_ts: req.end_ts,
            count: req.count,
            offset: req.offset,
            forward: req.forward,
            query: req.query,
        };
        let mut executor = self.query_executor.clone();
        let res = executor.execute(executor_req);
        match res {
            Ok(mut query_res) => {
                // convert query response into grpc response.
                let mut lines = Vec::new();
                for line in query_res.lines.drain(..) {
                    lines.push(api::LogLine {
                        line: line.line,
                        ts: line.ts,
                        app: line.app,
                    });
                }
                Ok(TonicResponse::new(api::QueryResponse { lines: lines }))
            }
            Err(err_msg) => Err(Status::new(Code::Internal, err_msg)),
        }
    }

    async fn partitions(
        &self,
        _: Request<api::Empty>,
    ) -> Result<TonicResponse<api::PartitionResponse>, Status> {
        match get_partitions(&self.partition_path) {
            Ok(partitions) => Ok(TonicResponse::new(api::PartitionResponse {
                partitions: partitions,
            })),
            Err(e) => Err(Status::new(Code::Internal, format!("{}", e))),
        }
    }

    async fn push(
        &self,
        req: Request<api::PushRequest>,
    ) -> Result<TonicResponse<api::Empty>, Status> {
        Ok(TonicResponse::new(api::Empty {}))
    }
}

// Depricating rest api.
// A struct which can store the state which it needs.
// #[derive(Clone)]
// struct PushHandler {
//     sender: Sender<IngesterRequest>,
//     count: Arc<AtomicI32>,
// }

// impl PushHandler {
//     fn new(sender: Sender<IngesterRequest>) -> PushHandler {
//         PushHandler {
//             sender: sender,
//             count: Arc::new(AtomicI32::new(0)),
//         }
//     }
// }

// impl Handler for PushHandler {
//     fn handle(mut self, mut state: State) -> Box<HandlerFuture> {
//         let fut = Body::take_from(&mut state)
//             .concat2()
//             .then(move |body| match body {
//                 Ok(body) => {
//                     let result = serde_json::from_slice::<PushRequest>(&body.to_vec());
//                     match result {
//                         Ok(req) => {
//                             block_on(async {
//                                 let (complete_sender, complete_receiver) = oneshot::channel();
//                                 let ingester_req = IngesterRequest::Push(IngesterPush {
//                                     push_request: req,
//                                     complete_signal: complete_sender,
//                                 });
//                                 self.sender.send(ingester_req).await;
//                                 let res = complete_receiver.await;
//                                 info!("completed singnal {:?}", res);
//                                 match res {
//                                     Err(e) => {
//                                         info!("yay {:?}", e);
//                                     }
//                                     _ => {
//                                         // Ok is the real value can be error or success,
//                                         // please validate it.
//                                     }
//                                 }
//                             });
//                             let res = create_empty_response(&state, StatusCode::OK);
//                             oldfuture::future::ok((state, res))
//                         }
//                         Err(e) => {
//                             let res = create_response(
//                                 &state,
//                                 StatusCode::NOT_ACCEPTABLE,
//                                 mime::TEXT_PLAIN,
//                                 format!("{}", e),
//                             );
//                             oldfuture::future::ok((state, res))
//                         }
//                     }
//                 }
//                 Err(e) => return oldfuture::future::err((state, e.into_handler_error())),
//             });
//         Box::new(fut)
//     }
// }

// impl RefUnwindSafe for PushHandler {}
// impl NewHandler for PushHandler {
//     type Instance = Self;

//     fn new_handler(&self) -> GothamResult<Self::Instance> {
//         Ok(self.clone())
//     }
// }

#[derive(Clone)]
struct HelloHandler {}
impl Handler for HelloHandler {
    fn handle(mut self, mut state: State) -> Box<HandlerFuture> {
        let res = format!("Hi from chola").into_response(&state);
        Box::new(oldfuture::future::ok((state, res)))
    }
}

impl NewHandler for HelloHandler {
    type Instance = Self;

    fn new_handler(&self) -> GothamResult<Self::Instance> {
        Ok(self.clone())
    }
}

struct QueryHandler {
    executor: QueryExecutor<rocks_store::RocksStore>,
}
impl QueryHandler {
    fn execute(&mut self, req: QueryRequest) -> Result<QueryResponse, String> {
        self.executor.execute(req)
    }
}

impl Handler for QueryHandler {
    fn handle(self, mut state: State) -> Box<HandlerFuture> {
        let mut executor = self.clone();
        let fut = Body::take_from(&mut state)
            .concat2()
            .then(move |body| match body {
                Ok(body) => {
                    let result = serde_json::from_slice::<QueryRequest>(&body.to_vec());
                    match result {
                        Ok(req) => match executor.execute(req) {
                            Ok(res) => {
                                let body = serde_json::to_string(&res)
                                    .expect("Failed to serialise to json");
                                let res = create_response(
                                    &state,
                                    StatusCode::CREATED,
                                    mime::APPLICATION_JSON,
                                    body,
                                );

                                return oldfuture::future::ok((state, res));
                            }
                            Err(e) => {
                                println!("{:?}", e);
                                let res = create_response(
                                    &state,
                                    StatusCode::NOT_ACCEPTABLE,
                                    mime::TEXT_PLAIN,
                                    format!("{}", e),
                                );
                                return oldfuture::future::ok((state, res));
                            }
                        },
                        Err(e) => return oldfuture::future::err((state, e.into_handler_error())),
                    }
                }
                Err(e) => {
                    let res = create_response(
                        &state,
                        StatusCode::NOT_ACCEPTABLE,
                        mime::TEXT_PLAIN,
                        format!("{}", e),
                    );
                    return oldfuture::future::ok((state, res));
                }
            });
        Box::new(fut)
    }
}
impl Clone for QueryHandler {
    fn clone(&self) -> QueryHandler {
        QueryHandler {
            executor: self.executor.clone(),
        }
    }
}

#[derive(Clone)]
pub struct PartitionHandler {
    pub partition_path: String,
}

/// get_partitions returns partitions list that has been ingesterd into
/// pathivu.
fn get_partitions(path: &String) -> Result<Vec<String>, failure::Error> {
    let path = Path::new(path).join("partition");
    create_dir_all(&path)?;
    let mut partitions = Vec::new();
    let dir = fs::read_dir(&path)?;
    for entry in dir {
        match entry {
            Ok(entry) => {
                partitions.push(entry.file_name().into_string().unwrap());
            }
            Err(e) => bail!("{}", e),
        }
    }
    return Ok(partitions);
}

impl PartitionHandler {
    pub fn partitions(&self) -> Result<PartitionRes, failure::Error> {
        let partitions = get_partitions(&self.partition_path)?;
        Ok(PartitionRes {
            partitions: partitions,
        })
    }
}

impl Handler for PartitionHandler {
    fn handle(mut self, mut state: State) -> Box<HandlerFuture> {
        match self.partitions() {
            Ok(res) => {
                let body = serde_json::to_string(&res).expect("Failed to serialise to json");
                let res =
                    create_response(&state, StatusCode::CREATED, mime::APPLICATION_JSON, body);
                return Box::new(oldfuture::future::ok((state, res)));
            }
            Err(e) => {
                let res = create_response(
                    &state,
                    StatusCode::NOT_ACCEPTABLE,
                    mime::TEXT_PLAIN,
                    format!("{}", e),
                );
                return Box::new(oldfuture::future::ok((state, res)));
            }
        }
    }
}

impl NewHandler for PartitionHandler {
    type Instance = Self;

    fn new_handler(&self) -> GothamResult<Self::Instance> {
        Ok(self.clone())
    }
}

impl RefUnwindSafe for QueryHandler {}

impl NewHandler for QueryHandler {
    type Instance = Self;

    fn new_handler(&self) -> GothamResult<Self::Instance> {
        Ok(self.clone())
    }
}

pub struct Server {}
impl Server {
    pub fn start() -> Result<(), failure::Error> {
        let cfg = Config {
            dir: "/home/schoolboy/cholalog".to_string(),
            max_segment_size: 100 << 10,
            max_index_size: 100 << 10,
            max_batch_size: 20,
        };
        let store = rocks_store::RocksStore::new(cfg.clone())?;

        info!("replaying segment files");
        let mut replayer = Replayer::new(cfg.clone(), store.clone());
        replayer.replay()?;
        drop(replayer);

        let (mut sender, receiver) = mpsc::channel(1000);
        let mut ingester = Ingester::new(receiver, cfg.clone(), store.clone());
        thread::spawn(move || {
            ingester.start();
        });
        let executor = QueryExecutor::new(cfg.clone(), sender.clone(), store);
        let addr = "0.0.0.0:6180".parse().unwrap();
        let pathivu_grpc = PathivuGrpcServer {
            ingester_transport: sender.clone(),
            query_executor: executor.clone(),
            partition_path: cfg.dir.clone(),
        };
        thread::spawn(move || {
            let mut rt = Runtime::new().unwrap();
            rt.block_on(async {
                println!("Listening pathivu grpc server on 6180");
                TonicServer::builder()
                    .add_service(PathivuServer::new(pathivu_grpc))
                    .serve(addr)
                    .await
                    .unwrap();
                (())
            });
        });
        info!("pathivu lisenting on 5180");
        let addr = "0.0.0.0:5180";
        println!("Listening for requests at http://{}", addr);
        let router = build_simple_router(|route| {
            //route.post("/push").to_new_handler(PushHandler::new(sender));
            route.get("/hello").to_new_handler(HelloHandler {});
            route
                .post("/query")
                .to_new_handler(QueryHandler { executor: executor });
            route.get("/partitions").to_new_handler(PartitionHandler {
                partition_path: cfg.dir,
            })
        });
        gotham::start(addr, router);
        Ok(())
    }
}
