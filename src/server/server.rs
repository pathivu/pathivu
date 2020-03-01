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
use crate::ingester::manager::Manager;
use crate::queryexecutor::executor::QueryExecutor;
use crate::replayer::replayer::Replayer;
use crate::store::rocks_store;
use crate::types::types::*;
use api::pathivu_server::{Pathivu, PathivuServer};
use failure::bail;
use future;
use futures::channel::mpsc;
use futures::channel::oneshot;
use futures::executor::block_on;
use futures::prelude::*;
use gotham;
use gotham::error::Result as GothamResult;
use gotham::handler::{Handler, HandlerFuture, IntoResponse, NewHandler};
use gotham::helpers::http::response::create_response;
use gotham::hyper::{body, Body, StatusCode};
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use log::info;
use mime;
use std::fs;
use std::fs::create_dir_all;
use std::panic::RefUnwindSafe;
use std::path::PathBuf;
use std::pin::Pin;
use std::thread;
use tokio::runtime::Runtime;
use tonic;
use tonic::{transport::Server as TonicServer, Code, Request, Response as TonicResponse, Status};
struct PathivuGrpcServer {
    ingester_manager: Manager,
    query_executor: QueryExecutor<rocks_store::RocksStore>,
    partition_path: PathBuf,
}

#[tonic::async_trait]
impl Pathivu for PathivuGrpcServer {
    type TailStream = mpsc::Receiver<Result<api::QueryResponse, Status>>;

    async fn tail(
        &self,
        req: Request<api::QueryRequest>,
    ) -> Result<TonicResponse<Self::TailStream>, Status> {
        let (tx, rx) = mpsc::channel(100);
        let mut req = req.into_inner();
        if req.partitions.len() == 0 {
            let partitions = get_partitions(&self.partition_path);
            match partitions {
                Ok(partitions) => req.partitions = partitions,
                Err(e) => {
                    return Err(Status::new(Code::Internal, format!("{}", e)));
                }
            }
        }
        let mut ingester_transport = self.ingester_manager.clone();
        if let Err(e) = ingester_transport.register_tailer(req.partitions, tx) {
            return Err(Status::new(Code::Internal, format!("{}", e)));
        }
        Ok(TonicResponse::new(rx))
    }

    async fn query(
        &self,
        req: Request<api::QueryRequest>,
    ) -> Result<TonicResponse<api::QueryResponse>, Status> {
        let req = req.into_inner();
        // convert into executor request.
        let mut executor = self.query_executor.clone();
        let res = executor.execute(req.query, req.start_ts, req.end_ts, req.forward, req.count);
        match res {
            Ok(query_res) => Ok(TonicResponse::new(api::QueryResponse {
                json: query_res,
                lines: Vec::default(),
            })),
            Err(err_msg) => Err(Status::new(Code::Internal, format!("{}", err_msg))),
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

    /// push will ingest log line.
    async fn push(
        &self,
        req: Request<api::PushRequest>,
    ) -> Result<TonicResponse<api::Empty>, Status> {
        let mut manager = self.ingester_manager.clone();
        if let Err(e) = block_on(async {
            let (complete_sender, complete_receiver) = oneshot::channel();
            let ingester_req = IngesterPush {
                push_request: req.into_inner(),
                complete_signal: complete_sender,
            };
            if let Err(e) = manager.ingest(ingester_req).await {
                return Err(format!("{}", e));
            }
            if let Err(e) = complete_receiver.await {
                return Err(format!("{}", e));
            }
            return Ok(());
        }) {
            return Err(Status::new(Code::Internal, e));
        }
        Ok(TonicResponse::new(api::Empty {}))
    }
}

#[derive(Clone)]
struct HelloHandler {}
impl Handler for HelloHandler {
    fn handle(self, state: State) -> Pin<Box<HandlerFuture>> {
        let res = format!("Hi from chola").into_response(&state);
        future::ok((state, res)).boxed()
    }
}

impl NewHandler for HelloHandler {
    type Instance = Self;

    fn new_handler(&self) -> GothamResult<Self::Instance> {
        Ok(self.clone())
    }
}

#[derive(Clone)]
struct PushHandler {
    manager: Manager,
}
impl NewHandler for PushHandler {
    type Instance = Self;

    fn new_handler(&self) -> GothamResult<Self::Instance> {
        Ok(self.clone())
    }
}
impl Handler for PushHandler {
    fn handle(self, mut state: State) -> Pin<Box<HandlerFuture>> {
        let mut manager = self.manager.clone();
        let fut =
            body::to_bytes(Body::take_from(&mut state)).then(move |full_body| match full_body {
                Ok(full_body) => {
                    let result = serde_json::from_slice::<PushRequest>(&full_body.to_vec());
                    match result {
                        Ok(req) => {
                            let mut lines = Vec::new();
                            for line in req.lines {
                                lines.push(api::PushLogLine {
                                    structured: line.structured,
                                    indexes: line.indexes,
                                    json_keys: line.json_keys,
                                    ts: line.ts,
                                    raw_data: line.raw_data.into_bytes(),
                                })
                            }
                            let push_req = api::PushRequest {
                                source: req.source,
                                lines: lines,
                            };
                            if let Err(e) = block_on(async {
                                let (complete_sender, complete_receiver) = oneshot::channel();
                                let ingester_req = IngesterPush {
                                    push_request: push_req,
                                    complete_signal: complete_sender,
                                };
                                if let Err(e) = manager.ingest(ingester_req).await {
                                    return Err(format!("{}", e));
                                }
                                if let Err(e) = complete_receiver.await {
                                    return Err(format!("{}", e));
                                }
                                return Ok(());
                            }) {
                                let mut res = create_response(
                                    &state,
                                    StatusCode::NOT_ACCEPTABLE,
                                    mime::TEXT_PLAIN,
                                    format!("{}", e),
                                );
                                let header = res.headers_mut();
                                header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                                return future::ok((state, res));
                            }
                        }
                        Err(e) => {
                            let mut res = create_response(
                                &state,
                                StatusCode::NOT_ACCEPTABLE,
                                mime::TEXT_PLAIN,
                                format!("{}", e),
                            );
                            let header = res.headers_mut();
                            header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                            return future::ok((state, res));
                        }
                    }
                    let mut res = create_response(
                        &state,
                        StatusCode::CREATED,
                        mime::TEXT_PLAIN,
                        format!("ok"),
                    );
                    let header = res.headers_mut();
                    header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                    return future::ok((state, res));
                }
                Err(e) => {
                    let mut res = create_response(
                        &state,
                        StatusCode::NOT_ACCEPTABLE,
                        mime::TEXT_PLAIN,
                        format!("{}", e),
                    );
                    let header = res.headers_mut();
                    header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                    return future::ok((state, res));
                }
            });
        fut.boxed()
    }
}

struct QueryHandler {
    executor: QueryExecutor<rocks_store::RocksStore>,
}
impl QueryHandler {
    fn execute(&mut self, req: QueryRequest) -> Result<String, failure::Error> {
        self.executor
            .execute(req.query, req.start_ts, req.end_ts, req.forward, req.count)
    }
}

impl Handler for QueryHandler {
    fn handle(self, mut state: State) -> Pin<Box<HandlerFuture>> {
        let mut executor = self.clone();
        let fut = body::to_bytes(Body::take_from(&mut state)).then(move |body| match body {
            Ok(body) => {
                let result = serde_json::from_slice::<QueryRequest>(&body.to_vec());
                match result {
                    Ok(req) => match executor.execute(req) {
                        Ok(res) => {
                            let mut res = create_response(
                                &state,
                                StatusCode::OK,
                                mime::APPLICATION_JSON,
                                res,
                            );
                            let header = res.headers_mut();
                            header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());

                            return future::ok((state, res));
                        }
                        Err(e) => {
                            let mut res = create_response(
                                &state,
                                StatusCode::NOT_ACCEPTABLE,
                                mime::TEXT_PLAIN,
                                format!("{}", e),
                            );
                            let header = res.headers_mut();
                            header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                            return future::ok((state, res));
                        }
                    },
                    Err(e) => {
                        let mut res = create_response(
                            &state,
                            StatusCode::NOT_ACCEPTABLE,
                            mime::TEXT_PLAIN,
                            format!("{}", e),
                        );
                        let header = res.headers_mut();
                        header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                        return future::ok((state, res));
                    }
                }
            }
            Err(e) => {
                let mut res = create_response(
                    &state,
                    StatusCode::NOT_ACCEPTABLE,
                    mime::TEXT_PLAIN,
                    format!("{}", e),
                );
                let header = res.headers_mut();
                header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                return future::ok((state, res));
            }
        });
        fut.boxed()
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
    pub partition_path: PathBuf,
}

/// get_partitions returns partitions list that has been ingesterd into
/// pathivu.
pub fn get_partitions(path: &PathBuf) -> Result<Vec<String>, failure::Error> {
    let path = path.join("partition");
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
    fn handle(self, state: State) -> Pin<Box<HandlerFuture>> {
        match self.partitions() {
            Ok(res) => {
                let body = serde_json::to_string(&res).expect("Failed to serialise to json");
                let mut res = create_response(&state, StatusCode::OK, mime::APPLICATION_JSON, body);
                let header = res.headers_mut();
                header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                return future::ok((state, res)).boxed();
            }
            Err(e) => {
                let mut res = create_response(
                    &state,
                    StatusCode::NOT_ACCEPTABLE,
                    mime::TEXT_PLAIN,
                    format!("{}", e),
                );
                let header = res.headers_mut();
                header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
                return future::ok((state, res)).boxed();
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

impl RefUnwindSafe for PushHandler {}

impl NewHandler for QueryHandler {
    type Instance = Self;

    fn new_handler(&self) -> GothamResult<Self::Instance> {
        Ok(self.clone())
    }
}

#[derive(Clone)]
struct CorsHandler {}

impl Handler for CorsHandler {
    fn handle(self, state: State) -> Pin<Box<HandlerFuture>> {
        let mut res = create_response(
            &state,
            StatusCode::OK,
            mime::APPLICATION_JSON,
            String::from(""),
        );
        let header = res.headers_mut();
        header.insert("Access-Control-Allow-Origin", "*".parse().unwrap());
        header.insert("Access-Control-Allow-Methods", "POST".parse().unwrap());
        header.insert(
            "Access-Control-Allow-Headers",
            "Content-Type".parse().unwrap(),
        );
        return future::ok((state, res)).boxed();
    }
}
impl NewHandler for CorsHandler {
    type Instance = Self;

    fn new_handler(&self) -> GothamResult<Self::Instance> {
        Ok(self.clone())
    }
}
pub struct Server {}
impl Server {
    pub fn start(cfg: Config, store: rocks_store::RocksStore) -> Result<(), failure::Error> {
        info!("replaying segment files");
        let mut replayer = Replayer::new(cfg.clone(), store.clone());
        replayer.replay()?;
        drop(replayer);

        let manager = Manager::new(cfg.clone(), store.clone());
        let executor = QueryExecutor::new(cfg.clone(), manager.clone(), store);
        let addr = cfg.grpc_addr.parse().unwrap();
        let pathivu_grpc = PathivuGrpcServer {
            ingester_manager: manager.clone(),
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

        println!("Listening for requests at http://{}", cfg.http_addr);
        let router = build_simple_router(|route| {
            route
                .post("/push")
                .to_new_handler(PushHandler { manager: manager });
            route.get("/hello").to_new_handler(HelloHandler {});
            route
                .post("/query")
                .to_new_handler(QueryHandler { executor: executor });
            route.options("/query").to_new_handler(CorsHandler {});
            route.get("/partitions").to_new_handler(PartitionHandler {
                partition_path: cfg.dir.clone(),
            })
        });
        gotham::start(cfg.http_addr, router);
        Ok(())
    }
}
