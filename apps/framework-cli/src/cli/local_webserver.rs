use super::display::Message;
use super::display::MessageType;

use crate::cli::routines::stop::StopLocalInfrastructure;
use crate::cli::routines::Routine;
use crate::cli::routines::RunMode;
use crate::framework::controller::RouteMeta;

use crate::framework::core::infrastructure::api_endpoint::APIType;
use crate::framework::core::infrastructure_map::ApiChange;
use crate::framework::core::infrastructure_map::Change;

use crate::framework::data_model::config::EndpointIngestionFormat;
use crate::infrastructure::stream::redpanda;
use crate::infrastructure::stream::redpanda::ConfiguredProducer;

use crate::project::Project;
use bytes::Buf;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use hyper_util::{rt::TokioExecutor, server::conn::auto};
use log::debug;
use log::error;
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalWebserverConfig {
    pub host: String,
    pub port: u16,
}

impl LocalWebserverConfig {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub fn url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

impl Default for LocalWebserverConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 4000,
        }
    }
}

async fn create_client(
    req: Request<hyper::body::Incoming>,
    host: String,
    consumption_apis: &RwLock<HashSet<String>>,
    is_prod: bool,
) -> Result<Response<Full<Bytes>>, anyhow::Error> {
    // local only for now
    let url = format!("http://{}:{}", host, 4001).parse::<hyper::Uri>()?;

    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap();
    let address = format!("{}:{}", host, port);
    let path = req.uri().to_string();
    let cleaned_path = path.strip_prefix("/consumption").unwrap_or(&path);

    debug!("Creating client for route: {:?}", cleaned_path);
    {
        let consumption_apis = consumption_apis.read().await;
        let consumption_name = req
            .uri()
            .path()
            .strip_prefix("/consumption/")
            .unwrap_or(cleaned_path);
        if !consumption_apis.contains(consumption_name) {
            if !is_prod {
                println!(
                    "Consumption API {} not found. Available consumption paths: {}",
                    consumption_name,
                    consumption_apis
                        .iter()
                        .map(|p| p.as_str())
                        .collect::<Vec<&str>>()
                        .join(", ")
                );
            }
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("Consumption API not found.")))
                .unwrap());
        }
    }

    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let authority = url.authority().unwrap().clone();

    let req = Request::builder()
        .uri(cleaned_path)
        .header(hyper::header::HOST, authority.as_str())
        .body(Full::new(Bytes::new()))?;

    let res = sender.send_request(req).await?;
    let body = res.collect().await.unwrap().to_bytes().to_vec();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Method", "GET, POST")
        .header("Access-Control-Allow-Headers", "Content-Type")
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

#[derive(Clone)]
struct RouteService {
    host: String,
    route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
    consumption_apis: &'static RwLock<HashSet<String>>,
    configured_producer: ConfiguredProducer,
    current_version: String,
    is_prod: bool,
}

impl Service<Request<Incoming>> for RouteService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Box::pin(router(
            req,
            self.current_version.clone(),
            self.route_table,
            self.consumption_apis,
            self.configured_producer.clone(),
            self.host.clone(),
            self.is_prod,
        ))
    }
}

fn options_route() -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "POST, OPTIONS")
        .header(
            "Access-Control-Allow-Headers",
            "Content-Type, Baggage, Sentry-Trace",
        )
        .body(Full::new(Bytes::from("Success")))
        .unwrap();

    Ok(response)
}

fn health_route() -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from("Success")))
        .unwrap();
    Ok(response)
}

fn bad_json_response(e: serde_json::Error) -> Response<Full<Bytes>> {
    show_message!(
        MessageType::Error,
        Message {
            action: "ERROR".to_string(),
            details: format!("Invalid JSON: {:?}", e),
        }
    );

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Full::new(Bytes::from("Invalid JSON")))
        .unwrap()
}

fn success_response(uri: String) -> Response<Full<Bytes>> {
    show_message!(
        MessageType::Success,
        Message {
            action: "SUCCESS".to_string(),
            details: uri.clone(),
        }
    );

    Response::new(Full::new(Bytes::from("SUCCESS")))
}

fn internal_server_error_response() -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::from("Error")))
        .unwrap()
}

async fn send_payload_to_topic(
    configured_producer: &ConfiguredProducer,
    topic_name: &str,
    payload: Value,
) -> Result<(i32, i64), (KafkaError, OwnedMessage)> {
    let payload = serde_json::to_vec(&payload).unwrap();

    log::debug!("Sending payload {:?} to topic: {}", payload, topic_name);

    configured_producer
        .producer
        .send(
            FutureRecord::to(topic_name)
                .key(topic_name) // This should probably be generated by the client that pushes data to the API
                .payload(payload.as_slice()),
            Timeout::After(Duration::from_secs(1)),
        )
        .await
}

async fn handle_json_req(
    configured_producer: &ConfiguredProducer,
    topic_name: &str,
    req: Request<Incoming>,
) -> Response<Full<Bytes>> {
    // TODO probably a refactor to be done here with the array json but it doesn't seem to be
    // straightforward to do it in a generic way.
    let url = req.uri().to_string();
    let body = req.collect().await.unwrap().aggregate();
    let parsed: Result<Value, serde_json::Error> = serde_json::from_reader(body.reader());
    // TODO add check that the payload has the proper schema

    if let Err(e) = parsed {
        return bad_json_response(e);
    }

    let res = send_payload_to_topic(configured_producer, topic_name, parsed.ok().unwrap()).await;
    if let Err((kafka_error, _)) = res {
        debug!(
            "Failed to deliver message to {} with error: {}",
            topic_name, kafka_error
        );
        return internal_server_error_response();
    }

    success_response(url)
}

async fn handle_json_array_body(
    configured_producer: &ConfiguredProducer,
    topic_name: &str,
    req: Request<Incoming>,
) -> Response<Full<Bytes>> {
    // TODO probably a refactor to be done here with the json but it doesn't seem to be
    // straighforward to do it in a generic way.
    let url = req.uri().to_string();
    let body = req.collect().await.unwrap().aggregate().reader();
    let parsed: Result<Vec<Value>, serde_json::Error> = serde_json::from_reader(body);
    if let Err(e) = parsed {
        return bad_json_response(e);
    }

    let mut res_arr = Vec::new();
    for payload in parsed.ok().unwrap() {
        let res = send_payload_to_topic(configured_producer, topic_name, payload).await;
        res_arr.push(res)
    }

    if res_arr.iter().any(|res| res.is_err()) {
        return internal_server_error_response();
    }

    success_response(url)
}

async fn ingest_route(
    req: Request<hyper::body::Incoming>,
    route: PathBuf,
    configured_producer: ConfiguredProducer,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    show_message!(
        MessageType::Info,
        Message {
            action: "POST".to_string(),
            details: route.to_str().unwrap().to_string().to_string(),
        }
    );

    match route_table.read().await.get(&route) {
        Some(route_meta) => match route_meta.format {
            EndpointIngestionFormat::Json => {
                Ok(handle_json_req(&configured_producer, &route_meta.topic_name, req).await)
            }
            EndpointIngestionFormat::JsonArray => {
                Ok(handle_json_array_body(&configured_producer, &route_meta.topic_name, req).await)
            }
        },
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from(
                "Please run `moose ls` to view your routes",
            ))),
    }
}

async fn router(
    req: Request<hyper::body::Incoming>,
    current_version: String,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    consumption_apis: &RwLock<HashSet<String>>,
    configured_producer: ConfiguredProducer,
    host: String,
    is_prod: bool,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    debug!(
        "HTTP Request Received: {:?}, with Route Table {:?}",
        req, route_table
    );

    let route_prefix = PathBuf::from("/");
    let route = PathBuf::from(req.uri().path())
        .strip_prefix(route_prefix)
        .unwrap()
        .to_path_buf()
        .clone();

    debug!(
        "Processing route: {:?}, with Route Table {:?}",
        route, route_table
    );

    let route_split = route.to_str().unwrap().split('/').collect::<Vec<&str>>();
    match (req.method(), &route_split[..]) {
        (&hyper::Method::POST, ["ingest", _]) => {
            ingest_route(
                req,
                // without explicit version, go to current project version
                route.join(current_version),
                configured_producer,
                route_table,
            )
            .await
        }
        (&hyper::Method::POST, ["ingest", _, _]) => {
            ingest_route(req, route, configured_producer, route_table).await
        }

        (&hyper::Method::GET, ["consumption", _rt]) => {
            match create_client(req, host, consumption_apis, is_prod).await {
                Ok(response) => Ok(response),
                Err(e) => {
                    debug!("Error: {:?}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(Bytes::from("Error")))
                }
            }
        }
        (&hyper::Method::GET, ["health"]) => health_route(),

        (&hyper::Method::OPTIONS, _) => options_route(),
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("no match"))),
    }
}

#[derive(Debug)]
pub struct Webserver {
    host: String,
    port: u16,
}

impl Webserver {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub async fn socket(&self) -> SocketAddr {
        tokio::net::lookup_host(format!("{}:{}", self.host, self.port))
            .await
            .unwrap()
            .next()
            .unwrap()
    }

    pub async fn spawn_api_update_listener(
        &self,
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
    ) -> mpsc::Sender<ApiChange> {
        log::info!("Spawning API update listener");

        let (tx, mut rx) = mpsc::channel::<ApiChange>(32);

        tokio::spawn(async move {
            while let Some(api_change) = rx.recv().await {
                let mut route_table = route_table.write().await;
                match api_change {
                    ApiChange::ApiEndpoint(Change::Added(api_endpoint)) => {
                        log::info!("Adding route: {:?}", api_endpoint.path);
                        match api_endpoint.api_type {
                            APIType::INGRESS { target_topic } => {
                                route_table.insert(
                                    api_endpoint.path.clone(),
                                    RouteMeta {
                                        format: api_endpoint.format.clone(),
                                        topic_name: target_topic,
                                    },
                                );
                            }
                            APIType::EGRESS => {
                                log::warn!("Egress API not supported yet")
                            }
                        }
                    }
                    ApiChange::ApiEndpoint(Change::Removed(api_endpoint)) => {
                        log::info!("Removing route: {:?}", api_endpoint.path);
                        route_table.remove(&api_endpoint.path);
                    }
                    ApiChange::ApiEndpoint(Change::Updated { before, after }) => {
                        match &after.api_type {
                            APIType::INGRESS { target_topic } => {
                                log::info!("Replacing route: {:?} with {:?}", before, after);

                                route_table.remove(&before.path);
                                route_table.insert(
                                    after.path.clone(),
                                    RouteMeta {
                                        format: after.format.clone(),
                                        topic_name: target_topic.clone(),
                                    },
                                );
                            }
                            APIType::EGRESS => {
                                log::warn!("Egress API not supported yet")
                            }
                        }
                    }
                }
            }
        });

        tx
    }

    // TODO - when we retire the the old core, we should remove routeTable from the start method and using only
    // the channel to update the routes
    pub async fn start(
        &self,
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
        consumption_apis: &'static RwLock<HashSet<String>>,
        project: Arc<Project>,
    ) {
        //! Starts the local webserver
        let socket = self.socket().await;

        // We create a TcpListener and bind it to {project.http_server_config.host} on port {project.http_server_config.port}
        let listener = TcpListener::bind(socket).await.unwrap();
        let producer = redpanda::create_producer(project.redpanda_config.clone());

        show_message!(
            MessageType::Success,
            Message {
                action: "Started".to_string(),
                details: "Webserver.\n\n".to_string(),
            }
        );

        if !project.is_production {
            show_message!(
                MessageType::Highlight,
                Message {
                    action: "Next Steps".to_string(),
                    details: format!("\n\n💻 Run the moose 👉 `ls` 👈 command for a bird's eye view of your application and infrastructure\n\n📥 Send Data to Moose\n\tYour local development server is running at: http://{}:{}/ingest\n", project.http_server_config.host.clone(), socket.port()),
                }
            );
        }

        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
        let mut sigint =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();

        let route_service = RouteService {
            host: self.host.clone(),
            route_table,
            consumption_apis,
            current_version: project.cur_version().to_string(),
            configured_producer: producer,
            is_prod: project.is_production,
        };

        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    let run_mode = RunMode::Explicit;
                    StopLocalInfrastructure::new(project.clone()).run(run_mode).unwrap().show();
                    std::process::exit(0);
                }
                _ = sigterm.recv() => {
                    let run_mode = RunMode::Explicit;
                    StopLocalInfrastructure::new(project.clone()).run(run_mode).unwrap().show();
                    std::process::exit(0);
                }
                listener_result = listener.accept() => {
                    let (stream, _) = listener_result.unwrap();
                    // Use an adapter to access something implementing `tokio::io` traits as if they implement
                    // `hyper::rt` IO traits.
                    let io = TokioIo::new(stream);

                    let route_service = route_service.clone();

                    // Spawn a tokio task to serve multiple connections concurrently
                    tokio::task::spawn(async move {
                        // Run this server for... forever!
                        if let Err(e) = auto::Builder::new(TokioExecutor::new()).serve_connection(
                                io,
                                route_service,
                            ).await {
                                error!("server error: {}", e);
                            }

                    });
                }
            }
        }
    }
}
