use super::display::Message;
use super::display::MessageType;
use super::routines::auth::validate_auth_token;
use crate::metrics::MetricEvent;

use crate::cli::display::with_spinner;
use crate::framework::controller::RouteMeta;

use crate::framework::core::infrastructure::api_endpoint::APIType;
use crate::framework::core::infrastructure_map::Change;
use crate::framework::core::infrastructure_map::{ApiChange, InfrastructureMap};
use crate::metrics::Metrics;
use crate::utilities::auth::{get_claims, validate_jwt};
use crate::utilities::docker;

use crate::framework::data_model::config::EndpointIngestionFormat;
use crate::infrastructure::stream::redpanda;
use crate::infrastructure::stream::redpanda::ConfiguredProducer;

use crate::framework::typescript::bin::CliMessage;
use crate::project::{JwtConfig, Project};
use bytes::Buf;
use chrono::Utc;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Body;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::header::HeaderValue;
use hyper::service::Service;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use hyper_util::{rt::TokioExecutor, server::conn::auto};
use log::error;
use log::{debug, log};
use rdkafka::error::KafkaError;
use rdkafka::message::OwnedMessage;
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use rdkafka::producer::{DeliveryFuture, FutureRecord};
use rdkafka::util::Timeout;
use serde::Serialize;
use serde::{Deserialize, Deserializer};
use serde_json::Deserializer as JsonDeserializer;
use tokio::spawn;

use crate::framework::data_model::model::DataModel;
use crate::utilities::validate_passthrough::{DataModelArrayVisitor, DataModelVisitor};
use lazy_static::lazy_static;
use log::Level::{Debug, Trace};
use std::collections::{HashMap, HashSet};
use std::env;
use std::env::VarError;
use std::future::Future;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::ops::Deref;
use std::path::PathBuf;
use std::pin::Pin;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

pub struct RouterRequest {
    req: Request<hyper::body::Incoming>,
    route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
}

fn default_management_port() -> u16 {
    5001
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalWebserverConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_management_port")]
    pub management_port: u16,
    pub path_prefix: Option<String>,
}

impl LocalWebserverConfig {
    pub fn url(&self) -> String {
        let base_url = format!("http://{}:{}", self.host, self.port);
        if let Some(prefix) = &self.path_prefix {
            format!("{}/{}", base_url, prefix.trim_matches('/'))
        } else {
            base_url
        }
    }

    pub fn normalized_path_prefix(&self) -> Option<String> {
        self.path_prefix.as_ref().map(|prefix| {
            let trimmed = prefix.trim_matches('/');
            if trimmed.is_empty() {
                prefix.to_string()
            } else {
                format!("/{}", trimmed)
            }
        })
    }
}

impl Default for LocalWebserverConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 4000,
            management_port: default_management_port(),
            path_prefix: None,
        }
    }
}

async fn create_client(
    req: Request<hyper::body::Incoming>,
    host: String,
    consumption_apis: &RwLock<HashSet<String>>,
    is_prod: bool,
) -> Result<Response<Full<Bytes>>, anyhow::Error> {
    // Extract the Authorization header and check the bearer token
    let auth_header = req.headers().get(hyper::header::AUTHORIZATION);

    // JWT config for consumption api is handled in user's api files
    if !check_authorization(auth_header, &MOOSE_CONSUMPTION_API_KEY, &None).await {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Full::new(Bytes::from(
                "Unauthorized: Invalid or missing token",
            )))?);
    }

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

    let mut new_req: Request<Full<Bytes>> = Request::builder()
        .uri(cleaned_path)
        .header(hyper::header::HOST, authority.as_str())
        .body(Full::new(Bytes::new()))?;

    let headers = new_req.headers_mut();
    for (key, value) in req.headers() {
        headers.insert(key, value.clone());
    }

    let res = sender.send_request(new_req).await?;
    let status = res.status();
    let body = res.collect().await.unwrap().to_bytes().to_vec();

    Ok(Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Method", "GET, POST")
        .header(
            "Access-Control-Allow-Headers",
            "Authorization, Content-Type",
        )
        .body(Full::new(Bytes::from(body)))
        .unwrap())
}

#[derive(Clone)]
struct RouteService {
    host: String,
    path_prefix: Option<String>,
    route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
    consumption_apis: &'static RwLock<HashSet<String>>,
    jwt_config: Option<JwtConfig>,
    configured_producer: ConfiguredProducer,
    current_version: String,
    is_prod: bool,
    metrics: Arc<Metrics>,
}
#[derive(Clone)]
struct ManagementService<I: InfraMapProvider + Clone> {
    path_prefix: Option<String>,
    is_prod: bool,
    metrics: Arc<Metrics>,
    infra_map: I,
}

impl Service<Request<Incoming>> for RouteService {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Box::pin(router(
            self.current_version.clone(),
            self.path_prefix.clone(),
            self.consumption_apis,
            self.jwt_config.clone(),
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
impl<I: InfraMapProvider + Clone + Send + 'static> Service<Request<Incoming>>
    for ManagementService<I>
{
    type Response = Response<Full<Bytes>>;
    type Error = hyper::http::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        Box::pin(management_router(
            self.path_prefix.clone(),
            self.is_prod,
            self.metrics.clone(),
            // here we're either cloning the reference or the RwLock
            self.infra_map.clone(),
            req,
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
            "Authorization, Content-Type, Baggage, Sentry-Trace",
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

async fn log_route(req: Request<Incoming>) -> Response<Full<Bytes>> {
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
        Err(e) => println!("Received unkn message: {:?}", e),
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from("")))
        .unwrap()
}

async fn metrics_log_route(req: Request<Incoming>, metrics: Arc<Metrics>) -> Response<Full<Bytes>> {
    debug!("Received metrics log route");

    let body = to_reader(req).await;
    let parsed: Result<MetricEvent, serde_json::Error> = serde_json::from_reader(body);
    debug!("Parsed metrics log route: {:?}", parsed);

    if let Ok(MetricEvent::StreamingFunctionEvent {
        count_in,
        count_out,
        bytes,
        function_name,
        timestamp,
    }) = parsed
    {
        metrics
            .send_metric_event(MetricEvent::StreamingFunctionEvent {
                timestamp,
                count_in,
                count_out,
                bytes,
                function_name: function_name.clone(),
            })
            .await;
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from("")))
        .unwrap()
}

async fn metrics_route(metrics: Arc<Metrics>) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from(
            metrics.get_metrics_registry_as_string().await,
        )))
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
        .body(Full::new(Bytes::from(format!("Invalid JSON: {}", e))))
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

fn route_not_found_response() -> hyper::http::Result<Response<Full<Bytes>>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from("no match")))
}

async fn send_payload_to_topic(
    configured_producer: &ConfiguredProducer,
    topic_name: &str,
    payload: Vec<u8>,
) -> Result<(i32, i64), (KafkaError, OwnedMessage)> {
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
    data_model: &DataModel,
    req: Request<Incoming>,
    jwt_config: &Option<JwtConfig>,
) -> Response<Full<Bytes>> {
    let auth_header = req.headers().get(hyper::header::AUTHORIZATION);
    let jwt_claims = get_claims(auth_header, jwt_config);

    // TODO probably a refactor to be done here with the array json but it doesn't seem to be
    // straightforward to do it in a generic way.
    let url = req.uri().to_string();
    let body = to_reader(req).await;

    let parsed = JsonDeserializer::from_reader(body).deserialize_any(&mut DataModelVisitor::new(
        &data_model.columns,
        jwt_claims.as_ref(),
    ));

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
    data_model: &DataModel,
    req: Request<Incoming>,
    jwt_config: &Option<JwtConfig>,
) -> Response<Full<Bytes>> {
    let auth_header = req.headers().get(hyper::header::AUTHORIZATION);
    let jwt_claims = get_claims(auth_header, jwt_config);

    // TODO probably a refactor to be done here with the json but it doesn't seem to be
    // straightforward to do it in a generic way.
    let url = req.uri().to_string();
    let number_of_bytes = req.body().size_hint().exact().unwrap();
    let body = to_reader(req).await;

    debug!(
        "starting to parse json array with length {} for {}",
        number_of_bytes, topic_name
    );
    let parsed = JsonDeserializer::from_reader(body).deserialize_seq(&mut DataModelArrayVisitor {
        inner: DataModelVisitor::new(&data_model.columns, jwt_claims.as_ref()),
    });

    debug!("parsed json array for {}", topic_name);

    if let Err(e) = parsed {
        return bad_json_response(e);
    }

    let mut res_arr: Vec<Result<OwnedDeliveryResult, KafkaError>> = Vec::new();
    let mut temp_res: Vec<Result<DeliveryFuture, KafkaError>> = Vec::new();

    for (count, payload) in parsed.ok().unwrap().into_iter().enumerate() {
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

async fn validate_token(token: Option<&str>, key: &str) -> bool {
    token.is_some_and(|t| validate_auth_token(t, key))
}

fn get_env_var(s: &str) -> Option<String> {
    match env::var(s) {
        Ok(env_var) => Some(env_var),
        Err(VarError::NotPresent) => None,
        Err(VarError::NotUnicode(_)) => panic!("Invalid key for {}, NotUnicode", s),
    }
}

// TODO should we move this to the project config?
// Since it automatically loads the env var and orverrides local file settings
//That way, a user can set dev variables easily and override them in prod with env vars.
lazy_static! {
    static ref MOOSE_CONSUMPTION_API_KEY: Option<String> = get_env_var("MOOSE_CONSUMPTION_API_KEY");
    static ref MOOSE_INGEST_API_KEY: Option<String> = get_env_var("MOOSE_INGEST_API_KEY");
}

async fn check_authorization(
    auth_header: Option<&HeaderValue>,
    api_key: &Option<String>,
    jwt_config: &Option<JwtConfig>,
) -> bool {
    let bearer_token = auth_header
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|header_str| header_str.strip_prefix("Bearer "));

    if let Some(config) = jwt_config.as_ref() {
        if config.enforce_on_all_ingest_apis {
            return validate_jwt(
                bearer_token,
                &config.secret,
                &config.issuer,
                &config.audience,
            );
        }
    }

    if let Some(key) = api_key.as_ref() {
        return validate_token(bearer_token, key).await;
    }

    true
}

async fn ingest_route(
    req: Request<hyper::body::Incoming>,
    route: PathBuf,
    configured_producer: ConfiguredProducer,
    route_table: &RwLock<HashMap<PathBuf, RouteMeta>>,
    is_prod: bool,
    jwt_config: Option<JwtConfig>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    show_message!(
        MessageType::Info,
        Message {
            action: "POST".to_string(),
            details: route.to_str().unwrap().to_string().to_string(),
        }
    );

    let auth_header = req.headers().get(hyper::header::AUTHORIZATION);

    if !check_authorization(auth_header, &MOOSE_INGEST_API_KEY, &jwt_config).await {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Full::new(Bytes::from(
                "Unauthorized: Invalid or missing token",
            )));
    }

    match route_table.read().await.get(&route) {
        Some(route_meta) => match route_meta.format {
            EndpointIngestionFormat::Json => Ok(handle_json_req(
                &configured_producer,
                &route_meta.topic_name,
                &route_meta.data_model,
                req,
                &jwt_config,
            )
            .await),
            EndpointIngestionFormat::JsonArray => Ok(handle_json_array_body(
                &configured_producer,
                &route_meta.topic_name,
                &route_meta.data_model,
                req,
                &jwt_config,
            )
            .await),
        },
        None => {
            if !is_prod {
                println!("Ingestion route {:?} not found.", route);
            }
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from(
                    "Please run `moose ls` to view your routes",
                )))
        }
    }
}

fn get_path_without_prefix(path: PathBuf, path_prefix: Option<String>) -> PathBuf {
    let path_without_prefix = if let Some(prefix) = path_prefix {
        path.strip_prefix(&prefix).unwrap_or(&path).to_path_buf()
    } else {
        path
    };

    path_without_prefix
        .strip_prefix("/")
        .unwrap_or(&path_without_prefix)
        .to_path_buf()
}

#[allow(clippy::too_many_arguments)]
async fn router(
    current_version: String,
    path_prefix: Option<String>,
    consumption_apis: &RwLock<HashSet<String>>,
    jwt_config: Option<JwtConfig>,
    configured_producer: ConfiguredProducer,
    host: String,
    is_prod: bool,
    metrics: Arc<Metrics>,
    request: RouterRequest,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let now = Instant::now();

    let req = request.req;
    let req_bytes = req.body().size_hint().exact().unwrap();

    let route_table = request.route_table;

    debug!(
        "-> HTTP Request: {:?} - {:?}",
        req.method(),
        req.uri().path(),
    );

    let route = get_path_without_prefix(PathBuf::from(req.uri().path()), path_prefix);
    let route_clone = route.clone();

    let metrics_method = req.method().to_string();

    let route_split = route.to_str().unwrap().split('/').collect::<Vec<&str>>();
    let res = match (req.method(), &route_split[..]) {
        (&hyper::Method::POST, ["ingest", _]) => {
            ingest_route(
                req,
                // without explicit version, go to current project version
                route.join(current_version),
                configured_producer,
                route_table,
                is_prod,
                jwt_config,
            )
            .await
        }
        (&hyper::Method::POST, ["ingest", _, _]) => {
            ingest_route(
                req,
                route,
                configured_producer,
                route_table,
                is_prod,
                jwt_config,
            )
            .await
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
        _ => route_not_found_response(),
    };

    let res_bytes = res.as_ref().unwrap().body().size_hint().exact().unwrap();
    let topic = route_table
        .read()
        .await
        .get(&route_clone)
        .map(|route_meta| route_meta.topic_name.clone())
        .unwrap_or_default();

    let metrics_clone = metrics.clone();
    let metrics_path = route_clone.clone().to_str().unwrap().to_string();
    let metrics_path_clone = metrics_path.clone();

    spawn(async move {
        if metrics_path_clone.starts_with("ingest/") {
            let _ = metrics_clone
                .send_metric_event(MetricEvent::IngestedEvent {
                    topic,
                    timestamp: Utc::now(),
                    count: 1,
                    bytes: req_bytes,
                    latency: now.elapsed(),
                    route: metrics_path.clone(),
                    method: metrics_method.clone(),
                })
                .await;
        }

        if metrics_path_clone.starts_with("consumption/") {
            let _ = metrics_clone
                .send_metric_event(MetricEvent::ConsumedEvent {
                    timestamp: Utc::now(),
                    count: 1,
                    latency: now.elapsed(),
                    bytes: res_bytes,
                    route: metrics_path.clone(),
                    method: metrics_method.clone(),
                })
                .await;
        }
    });

    res
}

const METRICS_LOGS_PATH: &str = "metrics-logs";

async fn management_router<I: InfraMapProvider>(
    path_prefix: Option<String>,
    is_prod: bool,
    metrics: Arc<Metrics>,
    infra_map: I,
    req: Request<Incoming>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let level = if req.uri().path().ends_with(METRICS_LOGS_PATH) {
        Trace // too many lines of log created without user interaction
    } else {
        Debug
    };
    log!(
        level,
        "-> HTTP Request: {:?} - {:?}",
        req.method(),
        req.uri().path(),
    );

    let route = get_path_without_prefix(PathBuf::from(req.uri().path()), path_prefix);

    let route = route.to_str().unwrap();
    let res = match (req.method(), route) {
        (&hyper::Method::POST, "logs") if !is_prod => Ok(log_route(req).await),
        (&hyper::Method::POST, METRICS_LOGS_PATH) => {
            Ok(metrics_log_route(req, metrics.clone()).await)
        }
        (&hyper::Method::GET, "metrics") => metrics_route(metrics.clone()).await,
        (&hyper::Method::GET, "infra-map") => {
            let res = infra_map.serialize().await.unwrap();

            hyper::Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(res)))
        }
        _ => route_not_found_response(),
    };

    res
}

#[derive(Debug)]
pub struct Webserver {
    host: String,
    port: u16,
    management_port: u16,
}

impl Webserver {
    pub fn new(host: String, port: u16, management_port: u16) -> Self {
        Self {
            host,
            port,
            management_port,
        }
    }

    async fn get_socket(&self, port: u16) -> SocketAddr {
        tokio::net::lookup_host(format!("{}:{}", self.host, port))
            .await
            .unwrap()
            .next()
            .unwrap()
    }
    pub async fn socket(&self) -> SocketAddr {
        self.get_socket(self.port).await
    }
    pub async fn management_socket(&self) -> SocketAddr {
        self.get_socket(self.management_port).await
    }

    pub async fn spawn_api_update_listener(
        &self,
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
        consumption_apis: &'static RwLock<HashSet<String>>,
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
                            APIType::INGRESS {
                                target_topic,
                                data_model,
                                format,
                            } => {
                                route_table.insert(
                                    api_endpoint.path.clone(),
                                    RouteMeta {
                                        format,
                                        data_model: data_model.unwrap(),
                                        topic_name: target_topic,
                                    },
                                );
                            }
                            APIType::EGRESS => {
                                consumption_apis
                                    .write()
                                    .await
                                    .insert(api_endpoint.path.to_string_lossy().to_string());
                            }
                        }
                    }
                    ApiChange::ApiEndpoint(Change::Removed(api_endpoint)) => {
                        log::info!("Removing route: {:?}", api_endpoint.path);
                        match api_endpoint.api_type {
                            APIType::INGRESS { .. } => {
                                route_table.remove(&api_endpoint.path);
                            }
                            APIType::EGRESS => {
                                consumption_apis
                                    .write()
                                    .await
                                    .remove(&api_endpoint.path.to_string_lossy().to_string());
                            }
                        }
                    }
                    ApiChange::ApiEndpoint(Change::Updated { before, after }) => {
                        match &after.api_type {
                            APIType::INGRESS {
                                target_topic,
                                data_model,
                                format,
                            } => {
                                log::info!("Replacing route: {:?} with {:?}", before, after);

                                route_table.remove(&before.path);
                                route_table.insert(
                                    after.path.clone(),
                                    RouteMeta {
                                        format: *format,
                                        data_model: data_model.as_ref().unwrap().clone(),
                                        topic_name: target_topic.clone(),
                                    },
                                );
                            }
                            APIType::EGRESS => {
                                // Nothing to do, we don't need to update the route table
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
    pub async fn start<I: InfraMapProvider + Clone + Send + 'static>(
        &self,
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
        consumption_apis: &'static RwLock<HashSet<String>>,
        infra_map: I,
        project: Arc<Project>,
        metrics: Arc<Metrics>,
    ) {
        //! Starts the local webserver
        let socket = self.socket().await;
        // We create a TcpListener and bind it to {project.http_server_config.host} on port {project.http_server_config.port}
        let listener = TcpListener::bind(socket)
            .await
            .unwrap_or_else(|e| handle_listener_err(socket.port(), e));

        let management_socket = self.management_socket().await;
        let management_listener = TcpListener::bind(management_socket)
            .await
            .unwrap_or_else(|e| handle_listener_err(management_socket.port(), e));

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
                    details: format!("\n\n💻 Run the moose 👉 `ls` 👈 command for a bird's eye view of your application and infrastructure\n\n📥 Send Data to Moose\n\tYour local development server is running at: {}/ingest\n", project.http_server_config.url()),
                }
            );
        }

        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
        let mut sigint =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();

        let route_service = RouteService {
            host: self.host.clone(),
            path_prefix: project.http_server_config.normalized_path_prefix(),
            route_table,
            consumption_apis,
            jwt_config: project.jwt.clone(),
            current_version: project.cur_version().to_string(),
            configured_producer: producer,
            is_prod: project.is_production,
            metrics: metrics.clone(),
        };
        let management_service = ManagementService {
            path_prefix: project.http_server_config.normalized_path_prefix(),
            is_prod: project.is_production,
            metrics,
            infra_map,
        };

        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    if !project.is_production {
                        with_spinner("Stopping containers", || {
                             let _ = docker::stop_containers(&project);
                        }, true);
                    }
                    std::process::exit(0);
                }
                _ = sigterm.recv() => {
                    if !project.is_production {
                        with_spinner("Stopping containers", || {
                            let _ = docker::stop_containers(&project);
                       }, true);
                    }
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
                listener_result = management_listener.accept() => {
                    let (stream, _) = listener_result.unwrap();
                    let io = TokioIo::new(stream);

                    let management_service = management_service.clone();

                    tokio::task::spawn(async move {
                        if let Err(e) = auto::Builder::new(TokioExecutor::new()).serve_connection(
                                io,
                                management_service,
                            ).await {
                                error!("server error: {}", e);
                            }

                    });
                }
            }
        }
    }
}

pub trait InfraMapProvider {
    fn serialize(&self) -> impl Future<Output = serde_json::error::Result<String>> + Send;
}
impl InfraMapProvider for &RwLock<InfrastructureMap> {
    async fn serialize(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self.read().await.deref())
    }
}
impl InfraMapProvider for &InfrastructureMap {
    async fn serialize(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self)
    }
}

fn handle_listener_err(port: u16, e: std::io::Error) -> ! {
    match e.kind() {
        ErrorKind::AddrInUse => {
            eprintln!("Port {} already in use.", port);
            exit(1)
        }
        _ => panic!("Failed to listen to port {}: {:?}", port, e),
    }
}
