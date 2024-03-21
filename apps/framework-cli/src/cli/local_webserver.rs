use super::display::Message;
use super::display::MessageType;

use crate::cli::routines::stop::StopLocalInfrastructure;
use crate::cli::routines::Routine;
use crate::cli::routines::RunMode;
use crate::framework::controller::RouteMeta;

use crate::infrastructure::stream::redpanda;
use crate::infrastructure::stream::redpanda::ConfiguredProducer;

use crate::infrastructure::console::ConsoleConfig;
use crate::project::Project;
use crate::project::PROJECT;
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
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteWebserverConfig {
    pub host: String,
    pub port: u16,
}

impl RemoteWebserverConfig {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub fn url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

impl Default for RemoteWebserverConfig {
    fn default() -> Self {
        Self {
            host: "34.82.14.129".to_string(),
            port: 4000,
        }
    }
}

#[derive(Clone)]
struct RouteService {
    route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
    configured_producer: ConfiguredProducer,
    console_config: ConsoleConfig,
    current_version: String,
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
            self.configured_producer.clone(),
            self.console_config.clone(),
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

async fn ingest_route(
    req: Request<hyper::body::Incoming>,
    route: PathBuf,
    configured_producer: ConfiguredProducer,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    console_config: ConsoleConfig,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    show_message!(
        MessageType::Info,
        Message {
            action: "POST".to_string(),
            details: route.to_str().unwrap().to_string().to_string(),
        }
    );

    match route_table.read().await.get(&route) {
        Some(route_meta) => {
            let is_curl = req.headers().get("User-Agent").map_or_else(
                || false,
                |user_agent| {
                    user_agent
                        .to_str()
                        .map_or_else(|_| false, |s| s.starts_with("curl"))
                },
            );

            let body = req.collect().await.unwrap().to_bytes().to_vec();
            match serde_json::from_slice::<serde::de::IgnoredAny>(&body) {
                Ok(_) => {}
                Err(e) => {
                    show_message!(
                        MessageType::Error,
                        Message {
                            action: "ERROR".to_string(),
                            details: format!("Invalid JSON: {:?}", e),
                        }
                    );
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Full::new(Bytes::from("Invalid JSON")));
                }
            }

            let topic_name = &route_meta.table_name;

            let res = configured_producer
                .producer
                .send(
                    FutureRecord::to(topic_name)
                        .key(topic_name) // This should probably be generated by the client that pushes data to the API
                        .payload(&body),
                    Timeout::After(Duration::from_secs(1)),
                )
                .await;

            match res {
                Ok(_) => {
                    show_message!(
                        MessageType::Success,
                        Message {
                            action: "SUCCESS".to_string(),
                            details: route.to_str().unwrap().to_string(),
                        }
                    );
                    let response_bytes = if is_curl {
                        Bytes::from(format!("Success! Go to http://localhost:{}/infrastructure/views to view your data!", console_config.host_port))
                    } else {
                        Bytes::from("SUCCESS")
                    };
                    Ok(Response::new(Full::new(response_bytes)))
                }
                Err(e) => {
                    debug!("Error: {:?}", e);
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(Bytes::from("Error")))
                }
            }
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from(
                "Please visit /console to view your routes",
            ))),
    }
}

async fn router(
    req: Request<hyper::body::Incoming>,
    current_version: String,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    configured_producer: ConfiguredProducer,
    console_config: ConsoleConfig,
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
                console_config,
            )
            .await
        }
        (&hyper::Method::POST, ["ingest", _, _]) => {
            ingest_route(req, route, configured_producer, route_table, console_config).await
        }
        (&hyper::Method::GET, ["health"]) => health_route(),

        (&hyper::Method::OPTIONS, _) => options_route(),
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("no match"))),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    pub async fn start(
        &self,
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
        project: Arc<Project>,
    ) {
        //! Starts the local webserver
        let socket = self.socket().await;

        // We create a TcpListener and bind it to {project.http_server_config.host} on port {project.http_server_config.port}
        let listener = TcpListener::bind(socket).await.unwrap();

        let producer = redpanda::create_producer(project.redpanda_config.clone());

        {
            show_message!(
            MessageType::Info,
            Message {
                action: "Started".to_string(),
                details: format!(" web server on port http://{}:{}. You'll use this to host and port to send data to your MooseJS app", project.http_server_config.host.clone(), socket.port()),
            }
        );
        }

        if !PROJECT.lock().unwrap().is_production {
            show_message!(
            MessageType::Info,
            Message {
                action: "Started".to_string(),
                details: format!(" console on port http://{}:{}. Check it out to get a bird's eye view of your application and infrastructure", project.http_server_config.host.clone(), project.console_config.host_port),
            }
        );
        }

        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
        let mut sigint =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();

        let route_service = RouteService {
            route_table,
            current_version: project.version().to_string(),
            configured_producer: producer,
            console_config: project.console_config.clone(),
        };

        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    let run_mode = RunMode::Explicit;
                    StopLocalInfrastructure::new(project.clone()).run(run_mode).unwrap();
                    std::process::exit(0);
                }
                _ = sigterm.recv() => {
                    let run_mode = RunMode::Explicit;
                    StopLocalInfrastructure::new(project.clone()).run(run_mode).unwrap();
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
