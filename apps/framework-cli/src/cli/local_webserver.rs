use super::display::Message;
use super::display::MessageType;

use crate::cli::routines::stop::StopLocalInfrastructure;
use crate::cli::routines::Routine;
use crate::cli::routines::RunMode;
use crate::framework::controller::RouteMeta;

use crate::framework::core::infrastructure::api_endpoint::APIType;
use crate::framework::core::infrastructure_map::ApiChange;
use crate::framework::core::infrastructure_map::Change;
use crate::metrics::Method;

use super::super::metrics::{Metrics, MetricsMessage};
use crate::framework::data_model::config::EndpointIngestionFormat;
use crate::infrastructure::stream::redpanda;
use crate::infrastructure::stream::redpanda::ConfiguredProducer;

use crate::framework::typescript::ts_node::CliMessage;
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
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use rdkafka::producer::{DeliveryFuture, FutureRecord};
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
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

pub struct MetricsDetails {
    timer: Instant,
    metrics_method: Method,
}

pub struct RouterRequest {
    req: Request<hyper::body::Incoming>,
    route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
}

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
    metrics: Arc<Metrics>,
    route: PathBuf,
    metrics_details: MetricsDetails,
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

    metrics
        .send_data(MetricsMessage::HTTPLatency((
            route.clone(),
            metrics_details.timer.elapsed(),
            metrics_details.metrics_method,
        )))
        .await;

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
    metrics: Arc<Metrics>,
}

impl Service<Request<Incoming>> for RouteService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Box::pin(router(
            self.current_version.clone(),
            self.consumption_apis,
            self.configured_producer.clone(),
            self.host.clone(),
            self.is_prod,
            self.metrics.clone(),
            RouterRequest {
                req,
                route_table: self.route_table,
            },
        ))
    }
}

async fn options_route(
    metrics: Arc<Metrics>,
    route: PathBuf,
    metrics_details: MetricsDetails,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
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

    metrics
        .send_data(MetricsMessage::HTTPLatency((
            route.clone(),
            metrics_details.timer.elapsed(),
            metrics_details.metrics_method,
        )))
        .await;

    Ok(response)
}

async fn health_route(
    metrics: Arc<Metrics>,
    route: PathBuf,
    metrics_details: MetricsDetails,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from("Success")))
        .unwrap();

    metrics
        .send_data(MetricsMessage::HTTPLatency((
            route.clone(),
            metrics_details.timer.elapsed(),
            metrics_details.metrics_method,
        )))
        .await;
    Ok(response)
}

async fn log_route(
    req: Request<Incoming>,
    metrics: Arc<Metrics>,
    route: PathBuf,
    metrics_details: MetricsDetails,
) -> Response<Full<Bytes>> {
    let body = to_reader(req).await;
    let parsed: Result<CliMessage, serde_json::Error> = serde_json::from_reader(body);
    match parsed {
        Ok(cli_message) => {
            let message = Message {
                action: cli_message.action,
                details: cli_message.message,
            };
            show_message!(cli_message.message_type, message);
        }
        Err(e) => println!("Received unknown message: {:?}", e),
    }

    metrics
        .send_data(MetricsMessage::HTTPLatency((
            route.clone(),
            metrics_details.timer.elapsed(),
            metrics_details.metrics_method,
        )))
        .await;

    Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from("")))
        .unwrap()
}

async fn metrics_route(
    metrics: Arc<Metrics>,
    route: PathBuf,
    metrics_details: MetricsDetails,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from(metrics.clone().receive_data().await)))
        .unwrap();

    metrics
        .send_data(MetricsMessage::HTTPLatency((
            route.clone(),
            metrics_details.timer.elapsed(),
            metrics_details.metrics_method,
        )))
        .await;

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

    debug!("Sending payload {:?} to topic: {}", payload, topic_name);

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

async fn to_reader(req: Request<Incoming>) -> bytes::buf::Reader<impl Buf + Sized> {
    req.collect().await.unwrap().aggregate().reader()
}

async fn handle_json_req(
    configured_producer: &ConfiguredProducer,
    topic_name: &str,
    req: Request<Incoming>,
) -> Response<Full<Bytes>> {
    // TODO probably a refactor to be done here with the array json but it doesn't seem to be
    // straightforward to do it in a generic way.
    let url = req.uri().to_string();
    let body = to_reader(req).await;
    let parsed: Result<Value, serde_json::Error> = serde_json::from_reader(body);
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

async fn wait_for_batch_complete(
    res_arr: &mut Vec<Result<OwnedDeliveryResult, KafkaError>>,
    temp_res: Vec<Result<DeliveryFuture, KafkaError>>,
) {
    for future_res in temp_res {
        match future_res {
            Ok(future) => match future.await {
                Ok(res) => res_arr.push(Ok(res)),
                Err(_) => res_arr.push(Err(KafkaError::Canceled)),
            },
            Err(e) => res_arr.push(Err(e)),
        }
    }
}

async fn handle_json_array_body(
    configured_producer: &ConfiguredProducer,
    topic_name: &str,
    req: Request<Incoming>,
) -> Response<Full<Bytes>> {
    // TODO probably a refactor to be done here with the json but it doesn't seem to be
    // straightforward to do it in a generic way.
    let url = req.uri().to_string();
    let body = to_reader(req).await;

    let parsed: Result<Vec<Value>, serde_json::Error> = serde_json::from_reader(body);
    if let Err(e) = parsed {
        return bad_json_response(e);
    }

    let mut res_arr: Vec<Result<OwnedDeliveryResult, KafkaError>> = Vec::new();
    let mut temp_res: Vec<Result<DeliveryFuture, KafkaError>> = Vec::new();

    for (count, payload) in parsed.ok().unwrap().into_iter().enumerate() {
        let payload = serde_json::to_vec(&payload).unwrap();

        debug!("Sending payload {:?} to topic: {}", payload, topic_name);
        let record = FutureRecord::to(topic_name)
            .key(topic_name) // This should probably be generated by the client that pushes data to the API
            .payload(payload.as_slice());
        temp_res.push(
            configured_producer
                .producer
                .send_result(record)
                .map_err(|(e, _)| e),
        );
        // ideally we want to use redpanda::send_with_back_pressure
        // but it does not report the error back
        if count % 1024 == 1023 {
            wait_for_batch_complete(&mut res_arr, temp_res).await;
            temp_res = Vec::new();
        }
    }
    wait_for_batch_complete(&mut res_arr, temp_res).await;

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
    metrics: Arc<Metrics>,
    metrics_details: MetricsDetails,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    show_message!(
        MessageType::Info,
        Message {
            action: "POST".to_string(),
            details: route.to_str().unwrap().to_string().to_string(),
        }
    );

    metrics
        .send_data(MetricsMessage::HTTPLatency((
            route.clone(),
            metrics_details.timer.elapsed(),
            metrics_details.metrics_method,
        )))
        .await;

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
    current_version: String,
    consumption_apis: &RwLock<HashSet<String>>,
    configured_producer: ConfiguredProducer,
    host: String,
    is_prod: bool,
    metrics: Arc<Metrics>,
    request: RouterRequest,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let now = Instant::now();

    let req = request.req;
    let route_table = request.route_table;

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

    let metrics_method = match *req.method() {
        hyper::Method::POST => Method::POST,
        hyper::Method::GET => Method::GET,
        hyper::Method::PUT => Method::PUT,
        hyper::Method::DELETE => Method::DELETE,
        hyper::Method::HEAD => Method::HEAD,
        hyper::Method::OPTIONS => Method::OPTIONS,
        hyper::Method::CONNECT => Method::CONNECT,
        hyper::Method::TRACE => Method::TRACE,
        hyper::Method::PATCH => Method::PATCH,
        //hyper::Method::EXTENSION(_) => Method::OTHER,
        // hyper::Method::ExtensionInline() => Method::OTHER,
        // hyper::Method::ExtensionAllocated(_) => Method::OTHER,
        _ => Method::OTHER,
    };

    let route_split = route.to_str().unwrap().split('/').collect::<Vec<&str>>();
    match (req.method(), &route_split[..]) {
        (&hyper::Method::POST, ["ingest", _]) => {
            ingest_route(
                req,
                // without explicit version, go to current project version
                route.join(current_version),
                configured_producer,
                route_table,
                metrics,
                MetricsDetails {
                    timer: now,
                    metrics_method,
                },
            )
            .await
        }
        (&hyper::Method::POST, ["ingest", _, _]) => {
            ingest_route(
                req,
                route,
                configured_producer,
                route_table,
                metrics,
                MetricsDetails {
                    timer: now,
                    metrics_method,
                },
            )
            .await
        }

        (&hyper::Method::GET, ["consumption", _rt]) => {
            match create_client(
                req,
                host,
                consumption_apis,
                is_prod,
                metrics,
                route.to_path_buf(),
                MetricsDetails {
                    timer: now,
                    metrics_method,
                },
            )
            .await
            {
                Ok(response) => Ok(response),
                Err(e) => {
                    debug!("Error: {:?}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(Bytes::from("Error")))
                }
            }
        }
        (&hyper::Method::POST, ["logs"]) if !is_prod => Ok(log_route(
            req,
            metrics,
            route,
            MetricsDetails {
                timer: now,
                metrics_method,
            },
        )
        .await),
        (&hyper::Method::GET, ["health"]) => {
            health_route(
                metrics,
                route,
                MetricsDetails {
                    timer: now,
                    metrics_method,
                },
            )
            .await
        }
        (&hyper::Method::GET, ["metrics"]) => {
            metrics_route(
                metrics,
                route,
                MetricsDetails {
                    timer: now,
                    metrics_method,
                },
            )
            .await
        }

        (&hyper::Method::OPTIONS, _) => {
            options_route(
                metrics,
                route,
                MetricsDetails {
                    timer: now,
                    metrics_method,
                },
            )
            .await
        }
        _ => {
            metrics
                .send_data(MetricsMessage::HTTPLatency((
                    route.clone(),
                    now.elapsed(),
                    metrics_method,
                )))
                .await;
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("no match")))
        }
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
        metrics: Arc<Metrics>,
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
            metrics: metrics.clone(),
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
                    let io = TokioIo::new(stream);

                    let route_service = route_service.clone();

                    tokio::task::spawn(async move {
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
