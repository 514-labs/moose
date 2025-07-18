/// # Local Webserver Module
///
/// This module provides a local HTTP server implementation for development and testing.
/// It handles API requests, routes them to the appropriate handlers, and manages
/// infrastructure changes.
///
/// The webserver has two main components:
/// - The main API server that handles data ingestion and API requests
/// - A management server that provides administrative endpoints
///
/// Key features include:
/// - Dynamic route handling based on the infrastructure map
/// - Authentication and authorization
/// - Metrics collection
/// - Health checks and monitoring
/// - OpenAPI documentation
/// - Integration with Kafka for message publishing
///
/// The webserver is configurable through the `LocalWebserverConfig` struct and
/// can be started in both development and production modes.
use super::display::{with_spinner, with_spinner_async, Message, MessageType};
use super::routines::auth::validate_auth_token;
use super::routines::scripts::terminate_all_workflows;
use super::settings::Settings;
use crate::infrastructure::redis::redis_client::RedisClient;
use crate::infrastructure::stream::kafka::models::KafkaStreamConfig;
use crate::metrics::MetricEvent;

use crate::framework::core::infrastructure::api_endpoint::APIType;
use crate::framework::core::infrastructure_map::Change;
use crate::framework::core::infrastructure_map::{ApiChange, InfrastructureMap};
use crate::framework::core::infrastructure_map::{InfraChanges, OlapChange, TableChange};
use crate::framework::versions::Version;
use crate::metrics::Metrics;
use crate::utilities::auth::{get_claims, validate_jwt};

use crate::infrastructure::stream::kafka;
use crate::infrastructure::stream::kafka::models::ConfiguredProducer;

use crate::framework::typescript::bin::CliMessage;
use crate::project::{JwtConfig, Project};
use crate::utilities::docker::DockerClient;
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
use log::{debug, log, trace};
use log::{error, info, warn};
use rdkafka::error::KafkaError;
use rdkafka::producer::future_producer::OwnedDeliveryResult;
use rdkafka::producer::{DeliveryFuture, FutureProducer, FutureRecord};
use reqwest::Client;
use serde::Serialize;
use serde::{Deserialize, Deserializer};
use serde_json::{json, Deserializer as JsonDeserializer, Value};
use tokio::spawn;

use crate::framework::data_model::model::DataModel;
use crate::utilities::validate_passthrough::{DataModelArrayVisitor, DataModelVisitor};
use hyper_util::server::graceful::GracefulShutdown;
use lazy_static::lazy_static;
use log::Level::{Debug, Trace};
use std::collections::{HashMap, HashSet};
use std::env;
use std::env::VarError;
use std::fs;
use std::future::Future;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::ops::Deref;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

use crate::framework::core::infra_reality_checker::InfraDiscrepancies;
use crate::framework::core::infrastructure::table::Table;
use crate::infrastructure::processes::process_registry::ProcessRegistries;

/// Request wrapper for router handling.
/// This struct combines the HTTP request with the route table for processing.
pub struct RouterRequest {
    req: Request<hyper::body::Incoming>,
    route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
}

/// Default management port for the webserver.
/// This is used when no management port is specified in the configuration.
fn default_management_port() -> u16 {
    5001
}

/// Metadata for an API route.
/// This struct contains information about the route, including the topic name,
/// format, and data model.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RouteMeta {
    /// The Kafka topic name associated with this route
    pub kafka_topic_name: String,
    /// The data model associated with this route
    pub data_model: DataModel,
    /// The Kafka topic name for failed ingestions
    pub dead_letter_queue: Option<String>,
    /// The version of the the api
    pub version: Option<Version>,
}

/// Configuration for the local webserver.
/// This struct contains settings for the webserver, including host, port,
/// management port, and path prefix.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalWebserverConfig {
    /// The host to bind the webserver to
    pub host: String,
    /// The port to bind the main API server to
    pub port: u16,
    /// The port to bind the management server to
    #[serde(default = "default_management_port")]
    pub management_port: u16,
    /// The port to bind the proxy server to (for consumption APIs)
    #[serde(default = "default_proxy_port")]
    pub proxy_port: u16,
    /// Optional path prefix for all routes
    pub path_prefix: Option<String>,
}

pub fn default_proxy_port() -> u16 {
    4001
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
                format!("/{trimmed}")
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
            proxy_port: default_proxy_port(),
            path_prefix: None,
        }
    }
}

#[tracing::instrument(skip(consumption_apis, req, is_prod), fields(uri = %req.uri(), method = %req.method(), headers = ?req.headers()))]
async fn get_consumption_api_res(
    http_client: Arc<Client>,
    req: Request<hyper::body::Incoming>,
    host: String,
    consumption_apis: &RwLock<HashSet<String>>,
    is_prod: bool,
    proxy_port: u16,
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

    let url = format!(
        "http://{}:{}{}{}",
        host,
        proxy_port,
        req.uri().path().strip_prefix("/consumption").unwrap_or(""),
        req.uri()
            .query()
            .map_or("".to_string(), |q| format!("?{q}"))
    );

    debug!("Creating client for route: {:?}", url);
    {
        let consumption_apis = consumption_apis.read().await;
        let consumption_name = req
            .uri()
            .path()
            .strip_prefix("/consumption/")
            .unwrap_or(req.uri().path());

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

    let mut client_req = reqwest::Request::new(req.method().clone(), url.parse()?);

    // Copy headers
    let headers = client_req.headers_mut();
    for (key, value) in req.headers() {
        headers.insert(key, value.clone());
    }

    // Send request
    let res = http_client.execute(client_req).await?;
    let status = res.status();
    let body = res.bytes().await?;

    let returned_response = Response::builder()
        .status(status)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST")
        .header(
            "Access-Control-Allow-Headers",
            "Authorization, Content-Type, baggage, sentry-trace, traceparent, tracestate",
        )
        .header("Content-Type", "application/json")
        .body(Full::new(body))
        .unwrap();

    Ok(returned_response)
}

#[derive(Clone)]
struct RouteService {
    host: String,
    path_prefix: Option<String>,
    route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
    consumption_apis: &'static RwLock<HashSet<String>>,
    jwt_config: Option<JwtConfig>,
    configured_producer: Option<ConfiguredProducer>,
    current_version: String,
    is_prod: bool,
    metrics: Arc<Metrics>,
    http_client: Arc<Client>,
    project: Arc<Project>,
    redis_client: Arc<RedisClient>,
}

#[derive(Clone)]
struct ManagementService<I: InfraMapProvider + Clone> {
    path_prefix: Option<String>,
    is_prod: bool,
    metrics: Arc<Metrics>,
    infra_map: I,
    openapi_path: Option<PathBuf>,
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
            self.http_client.clone(),
            RouterRequest {
                req,
                route_table: self.route_table,
            },
            self.project.clone(),
            self.redis_client.clone(),
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
            self.openapi_path.clone(),
            req,
        ))
    }
}

fn options_route() -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .header(
            "Access-Control-Allow-Headers",
            "Authorization, Content-Type, baggage, sentry-trace, traceparent, tracestate",
        )
        .body(Full::new(Bytes::from("Success")))
        .unwrap();

    Ok(response)
}

async fn health_route(
    project: &Project,
    redis_client: &Arc<RedisClient>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    // Check Redis connectivity
    let redis_healthy = redis_client.is_connected();

    // Check Redpanda/Kafka connectivity
    let kafka_healthy = match kafka::client::health_check(&project.redpanda_config).await {
        Ok(_) => true,
        Err(e) => {
            warn!("Health check: Redpanda unavailable: {}", e);
            false
        }
    };

    // Check ClickHouse connectivity
    let olap_client =
        crate::infrastructure::olap::clickhouse::create_client(project.clickhouse_config.clone());
    let clickhouse_healthy = match olap_client.client.query("SELECT 1").execute().await {
        Ok(_) => true,
        Err(e) => {
            warn!("Health check: ClickHouse unavailable: {}", e);
            false
        }
    };

    // Prepare healthy and unhealthy lists
    let mut healthy = Vec::new();
    let mut unhealthy = Vec::new();

    if redis_healthy {
        healthy.push("Redis")
    } else {
        unhealthy.push("Redis")
    }
    if kafka_healthy {
        healthy.push("Redpanda")
    } else {
        unhealthy.push("Redpanda")
    }
    if clickhouse_healthy {
        healthy.push("ClickHouse")
    } else {
        unhealthy.push("ClickHouse")
    }

    // Create JSON response
    let status = if unhealthy.is_empty() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let json_response = serde_json::to_string_pretty(&serde_json::json!({
        "healthy": healthy,
        "unhealthy": unhealthy
    }))
    .unwrap_or_else(|_| String::from("{\"error\":\"Failed to serialize response\"}"));

    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json_response)))
}

async fn admin_reality_check_route(
    req: Request<hyper::body::Incoming>,
    admin_api_key: &Option<String>,
    project: &Project,
    redis_client: &Arc<RedisClient>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let auth_header = req.headers().get(hyper::header::AUTHORIZATION);

    // Validate authentication
    if let Err(e) = validate_admin_auth(auth_header, admin_api_key).await {
        return e.to_response();
    }

    // Create OLAP client and reality checker
    let olap_client =
        crate::infrastructure::olap::clickhouse::create_client(project.clickhouse_config.clone());
    let reality_checker =
        crate::framework::core::infra_reality_checker::InfraRealityChecker::new(olap_client);

    // Load infrastructure map from Redis
    let infra_map = match InfrastructureMap::load_from_redis(redis_client).await {
        Ok(Some(map)) => map,
        Ok(None) => InfrastructureMap::default(),
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(format!(
                    "Failed to get infrastructure map: {e}"
                ))))
        }
    };

    // Perform reality check
    match reality_checker.check_reality(project, &infra_map).await {
        Ok(discrepancies) => {
            let response = serde_json::json!({
                "status": "success",
                "discrepancies": discrepancies
            });

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Full::new(Bytes::from(response.to_string())))?)
        }
        Err(e) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Full::new(Bytes::from(format!(
                "{{\"status\": \"error\", \"message\": \"{e}\"}}"
            ))))?),
    }
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
        Err(e) => println!("Received unknown message: {e:?}"),
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Full::new(Bytes::from("")))
        .unwrap()
}

async fn metrics_log_route(req: Request<Incoming>, metrics: Arc<Metrics>) -> Response<Full<Bytes>> {
    trace!("Received metrics log route");

    let body = to_reader(req).await;
    let parsed: Result<MetricEvent, serde_json::Error> = serde_json::from_reader(body);
    trace!("Parsed metrics log route: {:?}", parsed);

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

async fn openapi_route(
    is_prod: bool,
    openapi_path: Option<PathBuf>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    if is_prod {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::new(Bytes::from(
                "OpenAPI spec not available in production",
            )))
            .unwrap());
    }

    if let Some(path) = openapi_path {
        match fs::read_to_string(path) {
            Ok(contents) => Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/yaml")
                .body(Full::new(Bytes::from(contents)))
                .unwrap()),
            Err(_) => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("Failed to read OpenAPI spec file")))
                .unwrap()),
        }
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("OpenAPI spec file not found")))
            .unwrap())
    }
}

fn bad_json_response(e: serde_json::Error) -> Response<Full<Bytes>> {
    show_message!(
        MessageType::Error,
        Message {
            action: "ERROR".to_string(),
            details: format!("Invalid JSON: {e:?}"),
        }
    );

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Full::new(Bytes::from(format!("Invalid JSON: {e}"))))
        .unwrap()
}

fn success_response(data_model_name: &str) -> Response<Full<Bytes>> {
    show_message!(
        MessageType::Success,
        Message {
            action: "[POST]".to_string(),
            details: format!("Data received at ingest API sink for {data_model_name}"),
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

async fn to_reader(req: Request<Incoming>) -> bytes::buf::Reader<impl Buf + Sized> {
    req.collect().await.unwrap().aggregate().reader()
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

async fn send_to_kafka<T: Iterator<Item = Vec<u8>>>(
    producer: &FutureProducer,
    topic_name: &str,
    records: T,
) -> Vec<Result<OwnedDeliveryResult, KafkaError>> {
    let mut res_arr: Vec<Result<OwnedDeliveryResult, KafkaError>> = Vec::new();
    let mut temp_res: Vec<Result<DeliveryFuture, KafkaError>> = Vec::new();

    for (count, payload) in records.enumerate() {
        log::trace!("Sending payload {:?} to topic: {}", payload, topic_name);
        let record = FutureRecord::to(topic_name)
            .key(topic_name) // This should probably be generated by the client that pushes data to the API
            .payload(payload.as_slice());

        temp_res.push(producer.send_result(record).map_err(|(e, _)| e));
        // ideally we want to use kafka::send_with_back_pressure
        // but it does not report the error back
        if count % 1024 == 1023 {
            wait_for_batch_complete(&mut res_arr, temp_res).await;

            temp_res = Vec::new();
        }
    }
    wait_for_batch_complete(&mut res_arr, temp_res).await;
    res_arr
}

async fn handle_json_array_body(
    configured_producer: &ConfiguredProducer,
    topic_name: &str,
    data_model: &DataModel,
    dead_letter_queue: &Option<&str>,
    req: Request<Incoming>,
    jwt_config: &Option<JwtConfig>,
) -> Response<Full<Bytes>> {
    let auth_header = req.headers().get(hyper::header::AUTHORIZATION);
    let jwt_claims = get_claims(auth_header, jwt_config);

    let number_of_bytes = req.body().size_hint().exact().unwrap_or(0);
    let body = req.collect().await.unwrap().to_bytes();

    debug!(
        "starting to parse json array with length {} for {}",
        number_of_bytes, topic_name
    );
    let parsed = JsonDeserializer::from_slice(&body).deserialize_any(&mut DataModelArrayVisitor {
        inner: DataModelVisitor::new(&data_model.columns, jwt_claims.as_ref()),
    });

    debug!("parsed json array for {}", topic_name);

    if let Err(e) = parsed {
        if let Some(dlq) = dead_letter_queue {
            let objects = match serde_json::from_slice::<Value>(&body) {
                Ok(Value::Array(values)) => values
                    .into_iter()
                    .filter_map(|v| match v {
                        Value::Object(o) => Some(o),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                Ok(Value::Object(value)) => vec![value],
                _ => {
                    info!(
                        "Received payload for {} is not valid JSON objects or arrays. Not sending them to DLQ.",
                        topic_name
                    );
                    vec![]
                }
            };
            send_to_kafka(
                &configured_producer.producer,
                dlq,
                objects.into_iter().map(|original_record| {
                    serde_json::to_vec(&json!({
                        "originalRecord": original_record,
                        "errorMessage": e.to_string(),
                        "errorType": "ValidationError",
                        "failedAt": chrono::Utc::now().to_rfc3339(),
                        "source": "api",
                        "requestBody": String::from_utf8_lossy(&body),
                        "topic": topic_name,
                    }))
                    .unwrap()
                }),
            )
            .await;
        }
        warn!(
            "Bad JSON in request to topic {}: {}. Body: {:?}",
            topic_name, e, body
        );
        return bad_json_response(e);
    }
    let res_arr = send_to_kafka(
        &configured_producer.producer,
        topic_name,
        parsed.ok().unwrap().into_iter(),
    )
    .await;

    if res_arr.iter().any(|res| res.is_err()) {
        error!(
            "Internal server error sending to topic {}. Body: {:?}",
            topic_name, body
        );
        return internal_server_error_response();
    }

    success_response(&data_model.name)
}

async fn validate_token(token: Option<&str>, key: &str) -> bool {
    token.is_some_and(|t| validate_auth_token(t, key))
}

fn get_env_var(s: &str) -> Option<String> {
    match env::var(s) {
        Ok(env_var) => Some(env_var),
        Err(VarError::NotPresent) => None,
        Err(VarError::NotUnicode(_)) => panic!("Invalid key for {s}, NotUnicode"),
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
            details: route.to_str().unwrap().to_string(),
        }
    );

    debug!("Attempting to find route: {:?}", route);
    let route_table_read = route_table.read().await;
    debug!("Available routes: {:?}", route_table_read.keys());

    let auth_header = req.headers().get(hyper::header::AUTHORIZATION);

    if !check_authorization(auth_header, &MOOSE_INGEST_API_KEY, &jwt_config).await {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Full::new(Bytes::from(
                "Unauthorized: Invalid or missing token",
            )));
    }

    // Case-insensitive route matching
    let route_str = route.to_str().unwrap().to_lowercase();
    let matching_route = route_table_read
        .iter()
        .find(|(k, _)| k.to_str().unwrap_or("").to_lowercase().eq(&route_str));

    match matching_route {
        Some((_, route_meta)) => Ok(handle_json_array_body(
            &configured_producer,
            &route_meta.kafka_topic_name,
            &route_meta.data_model,
            &route_meta.dead_letter_queue.as_deref(),
            req,
            &jwt_config,
        )
        .await),
        None => {
            if !is_prod {
                println!(
                    "Ingestion route {:?} not found. Available routes: {:?}",
                    route,
                    route_table_read.keys()
                );
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
    configured_producer: Option<ConfiguredProducer>,
    host: String,
    is_prod: bool,
    metrics: Arc<Metrics>,
    http_client: Arc<Client>,
    request: RouterRequest,
    project: Arc<Project>,
    redis_client: Arc<RedisClient>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    let now = Instant::now();

    let req = request.req;
    let req_bytes = match req.body().size_hint().exact() {
        Some(bytes) => bytes,
        None => {
            debug!("Could not get exact size hint from request body");
            0 // Default to 0 if we can't get the exact size
        }
    };

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
    let res = match (configured_producer, req.method(), &route_split[..]) {
        (Some(configured_producer), &hyper::Method::POST, ["ingest", _]) => {
            if project.features.data_model_v2 {
                // For v2, find the latest version if no version specified
                let route_table_read = route_table.read().await;
                let base_path = route.to_str().unwrap();
                let mut latest_version: Option<&Version> = None;

                // First find matching routes, then get latest version
                for (path, meta) in route_table_read.iter() {
                    let path_str = path.to_str().unwrap();
                    if path_str.starts_with(base_path) {
                        if let Some(version) = &meta.version {
                            if latest_version.is_none() || version > latest_version.unwrap() {
                                latest_version = Some(version);
                            }
                        }
                    }
                }

                match latest_version {
                    // If latest version exists, use it
                    Some(version) => {
                        ingest_route(
                            req,
                            route.join(version.to_string()),
                            configured_producer,
                            route_table,
                            is_prod,
                            jwt_config,
                        )
                        .await
                    }
                    None => {
                        // Otherwise, try direct route
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
                }
            } else {
                // For v1, append current version as before
                ingest_route(
                    req,
                    route.join(&current_version),
                    configured_producer,
                    route_table,
                    is_prod,
                    jwt_config,
                )
                .await
            }
        }
        (Some(configured_producer), &hyper::Method::POST, ["ingest", _, _]) => {
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
        (_, &hyper::Method::POST, ["admin", "integrate-changes"]) => {
            admin_integrate_changes_route(
                req,
                &project.authentication.admin_api_key,
                &project,
                &redis_client,
            )
            .await
        }
        (_, &hyper::Method::POST, ["admin", "plan"]) => {
            admin_plan_route(req, &project.authentication.admin_api_key, &redis_client).await
        }
        (_, &hyper::Method::GET, ["consumption", _rt]) => {
            match get_consumption_api_res(
                http_client,
                req,
                host,
                consumption_apis,
                is_prod,
                project.http_server_config.proxy_port,
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
        (_, &hyper::Method::GET, ["health"]) => health_route(&project, &redis_client).await,
        (_, &hyper::Method::GET, ["admin", "reality-check"]) => {
            admin_reality_check_route(
                req,
                &project.authentication.admin_api_key,
                &project,
                &redis_client,
            )
            .await
        }
        (_, &hyper::Method::OPTIONS, _) => options_route(),
        _ => route_not_found_response(),
    };

    let res_bytes = res.as_ref().unwrap().body().size_hint().exact().unwrap();
    let topic = route_table
        .read()
        .await
        .get(&route_clone)
        .map(|route_meta| route_meta.kafka_topic_name.clone())
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

pub trait InfraMapProvider {
    fn serialize(&self) -> impl Future<Output = serde_json::error::Result<String>> + Send;
    fn as_infra_map<'a>(
        &'a self,
    ) -> Option<Box<dyn std::ops::Deref<Target = InfrastructureMap> + 'a>>;
}

impl InfraMapProvider for &RwLock<InfrastructureMap> {
    async fn serialize(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self.read().await.deref())
    }
    fn as_infra_map<'a>(
        &'a self,
    ) -> Option<Box<dyn std::ops::Deref<Target = InfrastructureMap> + 'a>> {
        // This is a little hacky, but works for our use case
        Some(Box::new(futures::executor::block_on(self.read())))
    }
}

impl InfraMapProvider for &InfrastructureMap {
    async fn serialize(&self) -> serde_json::error::Result<String> {
        serde_json::to_string(self)
    }
    fn as_infra_map<'a>(
        &'a self,
    ) -> Option<Box<dyn std::ops::Deref<Target = InfrastructureMap> + 'a>> {
        Some(Box::new(*self))
    }
}

async fn management_router<I: InfraMapProvider>(
    path_prefix: Option<String>,
    is_prod: bool,
    metrics: Arc<Metrics>,
    infra_map: I,
    openapi_path: Option<PathBuf>,
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
        // TODO: changes from admin/integrate-changes should apply here
        (&hyper::Method::GET, "infra-map") => {
            let accept_header = req
                .headers()
                .get(hyper::header::ACCEPT)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_ascii_lowercase();

            if accept_header.contains("application/protobuf") {
                if let Some(map_ref) = infra_map.as_infra_map() {
                    let bytes = map_ref.to_proto_bytes();
                    Ok(hyper::Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/protobuf")
                        .body(Full::new(Bytes::from(bytes)))
                        .unwrap())
                } else {
                    Ok(hyper::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(Bytes::from(
                            "Failed to access infrastructure map",
                        )))
                        .unwrap())
                }
            } else {
                match infra_map.serialize().await {
                    Ok(res) => Ok(hyper::Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .body(Full::new(Bytes::from(res)))
                        .unwrap()),
                    Err(_) => Ok(hyper::Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Full::new(Bytes::from(
                            "Failed to serialize infrastructure map",
                        )))
                        .unwrap()),
                }
            }
        }
        (&hyper::Method::GET, "openapi.yaml") => openapi_route(is_prod, openapi_path).await,
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
        project: Arc<Project>,
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
        consumption_apis: &'static RwLock<HashSet<String>>,
    ) -> mpsc::Sender<(InfrastructureMap, ApiChange)> {
        log::info!("Spawning API update listener");

        let (tx, mut rx) = mpsc::channel::<(InfrastructureMap, ApiChange)>(32);

        tokio::spawn(async move {
            while let Some((infra_map, api_change)) = rx.recv().await {
                let mut route_table = route_table.write().await;
                match api_change {
                    ApiChange::ApiEndpoint(Change::Added(api_endpoint)) => {
                        log::info!("Adding route: {:?}", api_endpoint.path);
                        match api_endpoint.api_type {
                            APIType::INGRESS {
                                target_topic_id,
                                dead_letter_queue,
                                data_model,
                            } => {
                                // This is not namespaced
                                let topic =
                                    infra_map.find_topic_by_id(&target_topic_id).unwrap_or_else(
                                        || panic!("Topic not found: {target_topic_id}"),
                                    );

                                // This is now a namespaced topic
                                let kafka_topic =
                                    KafkaStreamConfig::from_topic(&project.redpanda_config, topic);

                                route_table.insert(
                                    api_endpoint.path.clone(),
                                    RouteMeta {
                                        data_model: data_model.unwrap(),
                                        dead_letter_queue,
                                        kafka_topic_name: kafka_topic.name,
                                        version: api_endpoint.version,
                                    },
                                );
                            }
                            APIType::EGRESS { .. } => {
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
                            APIType::EGRESS { .. } => {
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
                                target_topic_id,
                                dead_letter_queue,
                                data_model,
                            } => {
                                log::info!("Replacing route: {:?} with {:?}", before, after);

                                let topic = infra_map
                                    .find_topic_by_id(target_topic_id)
                                    .expect("Topic not found");

                                let kafka_topic =
                                    KafkaStreamConfig::from_topic(&project.redpanda_config, topic);

                                route_table.remove(&before.path);
                                route_table.insert(
                                    after.path.clone(),
                                    RouteMeta {
                                        data_model: data_model.as_ref().unwrap().clone(),
                                        dead_letter_queue: dead_letter_queue.clone(),
                                        kafka_topic_name: kafka_topic.name,
                                        version: after.version,
                                    },
                                );
                            }
                            APIType::EGRESS { .. } => {
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
    #[allow(clippy::too_many_arguments)]
    pub async fn start<I: InfraMapProvider + Clone + Send + 'static>(
        &self,
        settings: &Settings,
        route_table: &'static RwLock<HashMap<PathBuf, RouteMeta>>,
        consumption_apis: &'static RwLock<HashSet<String>>,
        infra_map: I,
        project: Arc<Project>,
        metrics: Arc<Metrics>,
        openapi_path: Option<PathBuf>,
        process_registry: Arc<RwLock<ProcessRegistries>>,
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

        // Check if proxy port is available
        let proxy_socket = self.get_socket(project.http_server_config.proxy_port).await;
        TcpListener::bind(proxy_socket)
            .await
            .unwrap_or_else(|e| handle_listener_err(proxy_socket.port(), e));

        let producer = if project.features.streaming_engine {
            Some(kafka::client::create_producer(
                project.redpanda_config.clone(),
            ))
        } else {
            None
        };

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

        let http_client = Arc::new(reqwest::Client::new());

        let redis_client = RedisClient::new(project.name(), project.redis_config.clone())
            .await
            .expect("Failed to initialize Redis client");

        let route_service = RouteService {
            host: self.host.clone(),
            path_prefix: project.http_server_config.normalized_path_prefix(),
            route_table,
            consumption_apis,
            jwt_config: project.jwt.clone(),
            current_version: project.cur_version().to_string(),
            configured_producer: producer,
            is_prod: project.is_production,
            http_client,
            metrics: metrics.clone(),
            project: project.clone(),
            redis_client: Arc::new(redis_client),
        };

        let management_service = ManagementService {
            path_prefix: project.http_server_config.normalized_path_prefix(),
            is_prod: project.is_production,
            metrics,
            infra_map,
            openapi_path,
        };

        let graceful = GracefulShutdown::new();
        let conn_builder: &'static _ =
            Box::leak(Box::new(auto::Builder::new(TokioExecutor::new())));

        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    info!("SIGINT received, shutting down");
                    break; // break the loop and no more connections will be accepted
                }
                _ = sigterm.recv() => {
                    info!("SIGTERM received, shutting down");
                    break;
                }
                listener_result = listener.accept() => {
                    let (stream, _) = listener_result.unwrap();
                    let io = TokioIo::new(stream);

                    // Create a clone of route_service for each connection
                    // since hyper needs to own the service (it can't just borrow it)
                    let route_service = route_service.clone();
                    let conn = conn_builder.serve_connection(
                        io,
                        route_service.clone(),
                    );
                    let watched = graceful.watch(conn);
                    // Set server_label to "API" for the main API server. This label is used in error logging below.
                    let server_label = "API";
                    let port = socket.port();
                    let project_name = route_service.project.name().to_string();
                    let version = route_service.current_version.clone();
                    tokio::task::spawn(async move {
                        if let Err(e) = watched.await {
                            error!("server error on {} server (port {}): {} [project: {}, version: {}]", server_label, port, e, project_name, version);
                        }
                    });
                }
                listener_result = management_listener.accept() => {
                    let (stream, _) = listener_result.unwrap();
                    let io = TokioIo::new(stream);

                    let management_service = management_service.clone();

                    let conn = conn_builder.serve_connection(
                        io,
                        management_service,
                    );
                    let watched = graceful.watch(conn);
                    // Set server_label to "Management" for the management server. This label is used in error logging below.
                    let server_label = "Management";
                    let port = management_socket.port();
                    let project_name = project.name().to_string();
                    let version = project.cur_version().to_string();
                    tokio::task::spawn(async move {
                        if let Err(e) = watched.await {
                            error!("server error on {} server (port {}): {} [project: {}, version: {}]", server_label, port, e, project_name, version);
                        }
                    });
                }
            }
        }

        shutdown(settings, &project, graceful, process_registry).await;
    }
}

fn handle_listener_err(port: u16, e: std::io::Error) -> ! {
    match e.kind() {
        ErrorKind::AddrInUse => {
            eprintln!(
                "Port {port} already in use. Terminate the process using that port and try again."
            );
            std::process::exit(1)
        }
        _ => panic!("Failed to listen to port {port}: {e:?}"),
    }
}
async fn shutdown(
    settings: &Settings,
    project: &Project,
    graceful: GracefulShutdown,
    process_registry: Arc<RwLock<ProcessRegistries>>,
) -> ! {
    // First, initiate the graceful shutdown of HTTP connections
    let shutdown_future = graceful.shutdown();

    // Wait for connections to close with a timeout
    tokio::select! {
        _ = shutdown_future => {
            info!("all connections gracefully closed");
        },
        _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
            warn!("timed out wait for all connections to close");
        }
    }

    // Stop all managed processes using the existing process registry
    let mut process_registry = process_registry.write().await;
    match process_registry.stop().await {
        Ok(_) => {
            info!("Successfully stopped all managed processes");
            super::display::show_message_wrapper(
                MessageType::Success,
                Message {
                    action: "Shutdown".to_string(),
                    details: "All processes stopped successfully".to_string(),
                },
            );
        }
        Err(e) => {
            error!("Failed to stop some managed processes: {}", e);
            super::display::show_message_wrapper(
                MessageType::Error,
                Message {
                    action: "Shutdown".to_string(),
                    details: format!("Failed to stop all processes: {e}"),
                },
            );
        }
    }

    // Shutdown the Docker containers if needed
    if !project.is_production {
        if project.features.workflows {
            super::display::show_message_wrapper(
                MessageType::Highlight,
                Message {
                    action: "Shutdown".to_string(),
                    details: "Stopping workflows...".to_string(),
                },
            );

            let termination_result = with_spinner_async(
                "Stopping all workflows",
                async { terminate_all_workflows(project).await },
                true,
            )
            .await;
            info!("Workflow termination result: {:?}", termination_result);

            match termination_result {
                Ok(_) => super::display::show_message_wrapper(
                    MessageType::Success,
                    Message {
                        action: "Shutdown".to_string(),
                        details: "All workflows stopped successfully".to_string(),
                    },
                ),
                Err(_) => super::display::show_message_wrapper(
                    MessageType::Error,
                    Message {
                        action: "Shutdown".to_string(),
                        details: "Failed to stop all workflows".to_string(),
                    },
                ),
            };
        }

        // Use the centralized settings function to check if containers should be shutdown
        let should_shutdown_containers = settings.should_shutdown_containers();

        // Only shutdown containers if this instance is responsible for infra
        if should_shutdown_containers && project.should_load_infra() {
            // Create docker client with a fresh settings reference
            let docker = DockerClient::new(settings);
            info!("Starting container shutdown process");

            // First display a clear message to the user
            super::display::show_message_wrapper(
                MessageType::Highlight,
                Message {
                    action: "Shutdown".to_string(),
                    details: "Stopping containers...".to_string(),
                },
            );

            with_spinner(
                "Stopping containers",
                || {
                    let _ = docker.stop_containers(project);
                },
                true,
            );

            super::display::show_message_wrapper(
                MessageType::Success,
                Message {
                    action: "Shutdown".to_string(),
                    details: "All containers stopped successfully".to_string(),
                },
            );

            info!("Container shutdown complete");
        } else if !project.should_load_infra() {
            info!("Skipping container shutdown: load_infra is set to false for this instance");
        } else {
            info!("Skipping container shutdown due to settings configuration");
        }
    }

    // Final delay before exit to ensure any remaining tasks complete
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Exit the process cleanly
    info!("Exiting application");

    // Clear terminal using crossterm
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine)
    )
    .unwrap();

    std::process::exit(0);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrateChangesRequest {
    pub tables: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrateChangesResponse {
    pub status: String,
    pub message: String,
    pub updated_tables: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
enum IntegrationError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Internal Error: {0}")]
    InternalError(String),
}

impl IntegrationError {
    fn to_response(&self) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
        match self {
            IntegrationError::Unauthorized(msg) => Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Full::new(Bytes::from(msg.clone()))),
            IntegrationError::BadRequest(msg) => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(msg.clone()))),
            IntegrationError::InternalError(msg) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(msg.clone()))),
        }
    }
}

/// Validates the admin authentication by checking the provided bearer token against the admin API key.
///
/// # Arguments
/// * `auth_header` - Optional HeaderValue containing the Authorization header
/// * `admin_api_key` - Optional String containing the configured admin API key
///
/// # Returns
/// * `Ok(())` if authentication is successful
/// * `Err(IntegrationError)` if authentication fails or admin API key is not configured
async fn validate_admin_auth(
    auth_header: Option<&HeaderValue>,
    admin_api_key: &Option<String>,
) -> Result<(), IntegrationError> {
    debug!("Validating admin authentication");
    let bearer_token = auth_header
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|header_str| header_str.strip_prefix("Bearer "));

    if let Some(key) = admin_api_key {
        if !validate_token(bearer_token, key).await {
            debug!("Token validation failed");
            return Err(IntegrationError::Unauthorized(
                "Unauthorized: Invalid or missing token".to_string(),
            ));
        }
        debug!("Token validation successful");
        Ok(())
    } else {
        debug!("No admin API key configured");
        Err(IntegrationError::Unauthorized(
            "Unauthorized: Admin API key not configured".to_string(),
        ))
    }
}

/// Searches for a table definition in the provided discrepancies based on the table name.
/// This function looks for the table in unmapped tables, added tables, updated tables, and removed tables.
///
/// # Arguments
/// * `table_name` - Name of the table to find
/// * `discrepancies` - InfraDiscrepancies containing the differences between reality and infrastructure map
///
/// # Returns
/// * `Some(Table)` if the table definition is found
/// * `None` if the table is not found or is marked for removal
fn find_table_definition(table_name: &str, discrepancies: &InfraDiscrepancies) -> Option<Table> {
    debug!("Looking for table definition: {}", table_name);

    if discrepancies
        .unmapped_tables
        .iter()
        .any(|table| table.name == table_name)
    {
        debug!(
            "Table {} is unmapped, looking for its definition",
            table_name
        );
        // First try to find it in unmapped_tables
        if let Some(table) = discrepancies
            .unmapped_tables
            .iter()
            .find(|table| table.name == table_name)
        {
            return Some(table.clone());
        }
        // If not found in unmapped_tables, look for added tables
        discrepancies
            .mismatched_tables
            .iter()
            .find(|change| matches!(change, OlapChange::Table(TableChange::Added(table)) if table.name == table_name))
            .and_then(|change| match change {
                OlapChange::Table(TableChange::Added(table)) => Some(table.clone()),
                _ => None,
            })
    } else {
        debug!("Table {} is mapped, checking for updates", table_name);
        // Look for updated or removed tables
        match discrepancies
            .mismatched_tables
            .iter()
            .find(|change| match change {
                OlapChange::Table(TableChange::Updated { before, .. }) => before.name == table_name,
                OlapChange::Table(TableChange::Removed(table)) => table.name == table_name,
                _ => false,
            }) {
            Some(OlapChange::Table(TableChange::Updated { before, .. })) => {
                debug!("Found updated definition for table {}", table_name);
                Some(before.clone())
            }
            Some(OlapChange::Table(TableChange::Removed(_))) => {
                debug!("Table {} is marked for removal", table_name);
                None
            }
            _ => {
                debug!("No changes found for table {}", table_name);
                None
            }
        }
    }
}

/// Updates the infrastructure map with the provided tables based on the discrepancies.
/// This function handles adding new tables, updating existing ones, and removing tables as needed.
///
/// # Arguments
/// * `tables_to_update` - Vector of table names to update
/// * `discrepancies` - InfraDiscrepancies containing the differences between reality and infrastructure map
/// * `infra_map` - Mutable reference to the infrastructure map to update
///
/// # Returns
/// * Vector of strings containing the names of tables that were successfully updated
async fn update_inframap_tables(
    tables_to_update: Vec<String>,
    discrepancies: &InfraDiscrepancies,
    infra_map: &mut InfrastructureMap,
) -> Vec<String> {
    debug!("Updating inframap tables");
    let mut updated_tables = Vec::new();

    for table_name in tables_to_update {
        debug!("Processing table: {}", table_name);

        match find_table_definition(&table_name, discrepancies) {
            Some(table) => {
                debug!("Updating table {} in inframap", table_name);
                // Use table.id() as the key for the HashMap
                infra_map.tables.insert(table.id(), table);
                updated_tables.push(table_name);
            }
            None => {
                // When removing a table, we need to find its ID from the existing tables
                if discrepancies.mismatched_tables.iter().any(|change| {
                    matches!(change, OlapChange::Table(TableChange::Removed(table)) if table.name == table_name)
                }) {
                    debug!("Removing table {} from inframap", table_name);
                    // Find the table ID from the mismatched_tables
                    if let Some(OlapChange::Table(TableChange::Removed(table))) = discrepancies
                        .mismatched_tables
                        .iter()
                        .find(|change| matches!(change, OlapChange::Table(TableChange::Removed(table)) if table.name == table_name))
                    {
                        infra_map.tables.remove(&table.id());
                        updated_tables.push(table_name);
                    }
                } else {
                    debug!("No changes needed for table {}", table_name);
                    // Check if this table is in unmapped_tables
                    if let Some(table) = discrepancies.unmapped_tables.iter().find(|t| t.name == table_name) {
                        debug!("Found unmapped table {}, adding to inframap", table_name);
                        infra_map.tables.insert(table.id(), table.clone());
                        updated_tables.push(table_name);
                    } else {
                        debug!("Table {} is not unmapped", table_name);
                    }
                }
            }
        }
    }

    debug!("Updated {} tables", updated_tables.len());
    updated_tables
}

/// Stores the updated infrastructure map in both Redis and ClickHouse.
///
/// # Arguments
/// * `infra_map` - Reference to the infrastructure map to store
/// * `redis_guard` - Reference to the Redis client
/// * `project` - Reference to the project configuration
///
/// # Returns
/// * `Ok(())` if storage is successful
/// * `Err(IntegrationError)` if storage fails in either Redis or ClickHouse
async fn store_updated_inframap(
    infra_map: &InfrastructureMap,
    redis_client: Arc<RedisClient>,
) -> Result<(), IntegrationError> {
    debug!("Storing updated inframap");

    // Store in Redis
    if let Err(e) = infra_map.store_in_redis(&redis_client).await {
        debug!("Failed to store inframap in Redis: {}", e);
        return Err(IntegrationError::InternalError(format!(
            "Failed to store updated inframap in Redis: {e}"
        )));
    }
    debug!("Successfully stored inframap in Redis");

    Ok(())
}

/// Handles the admin integration changes route, which allows administrators to integrate
/// infrastructure changes into the system. This route validates authentication, processes
/// the requested table changes, and updates both the in-memory infrastructure map and
/// persisted storage (Redis and ClickHouse).
///
/// # Arguments
/// * `req` - The incoming HTTP request
/// * `admin_api_key` - Optional admin API key for authentication
/// * `project` - Reference to the project configuration
/// * `redis_client` - Reference to the Redis client wrapped in Arc<>
///
/// # Returns
/// * Result containing the HTTP response with either success or error information
async fn admin_integrate_changes_route(
    req: Request<hyper::body::Incoming>,
    admin_api_key: &Option<String>,
    project: &Project,
    redis_client: &Arc<RedisClient>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    debug!("Starting admin_integrate_changes_route");

    // Validate authentication
    if let Err(e) = validate_admin_auth(
        req.headers().get(hyper::header::AUTHORIZATION),
        admin_api_key,
    )
    .await
    {
        return e.to_response();
    }

    // Parse request body
    let body = to_reader(req).await;
    let request: IntegrateChangesRequest =
        match serde_json::from_reader::<_, IntegrateChangesRequest>(body) {
            Ok(req) => {
                debug!(
                    "Successfully parsed request body. Tables to integrate: {:?}",
                    req.tables
                );
                req
            }
            Err(e) => {
                debug!("Failed to parse request body: {}", e);
                return IntegrationError::BadRequest(format!("Invalid request body: {e}"))
                    .to_response();
            }
        };

    // Get reality check
    let olap_client =
        crate::infrastructure::olap::clickhouse::create_client(project.clickhouse_config.clone());
    let reality_checker =
        crate::framework::core::infra_reality_checker::InfraRealityChecker::new(olap_client);

    let mut infra_map = match InfrastructureMap::load_from_redis(redis_client).await {
        Ok(Some(infra_map)) => infra_map,
        Ok(None) => InfrastructureMap::default(),
        Err(e) => {
            return IntegrationError::InternalError(format!(
                "Failed to load infrastructure map: {e}"
            ))
            .to_response();
        }
    };

    let discrepancies = match reality_checker.check_reality(project, &infra_map).await {
        Ok(d) => d,
        Err(e) => {
            return IntegrationError::InternalError(format!("Failed to check reality: {e}"))
                .to_response();
        }
    };

    // Update tables in inframap
    let updated_tables =
        update_inframap_tables(request.tables, &discrepancies, &mut infra_map).await;

    if updated_tables.is_empty() {
        return IntegrationError::BadRequest(
            "None of the specified tables were found in reality check discrepancies".to_string(),
        )
        .to_response();
    }

    // Store updated inframap
    match store_updated_inframap(&infra_map, redis_client.clone()).await {
        Ok(_) => (),
        Err(e) => {
            return IntegrationError::InternalError(format!(
                "Failed to store updated inframap: {e}"
            ))
            .to_response();
        }
    };

    // Prepare success response
    let response = IntegrateChangesResponse {
        status: "success".to_string(),
        message: "Successfully integrated changes into inframap".to_string(),
        updated_tables,
    };

    debug!("Preparing success response: {:?}", response);
    match serde_json::to_string(&response) {
        Ok(json) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(json))),
        Err(e) => IntegrationError::InternalError(format!("Failed to serialize response: {e}"))
            .to_response(),
    }
}

/// Request structure for the admin plan endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct PlanRequest {
    // The client-generated inframap
    pub infra_map: InfrastructureMap,
}

/// Response structure for the admin plan endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct PlanResponse {
    pub status: String,
    // Changes that would be applied if the plan is executed
    pub changes: InfraChanges,
}

/// Handles the admin plan endpoint, which compares a submitted infrastructure map
/// with the server's current state and returns the changes that would be applied
async fn admin_plan_route(
    req: Request<hyper::body::Incoming>,
    admin_api_key: &Option<String>,
    redis_client: &Arc<RedisClient>,
) -> Result<Response<Full<Bytes>>, hyper::http::Error> {
    // Validate admin authentication
    let auth_header = req.headers().get(hyper::header::AUTHORIZATION);
    if let Err(e) = validate_admin_auth(auth_header, admin_api_key).await {
        return e.to_response();
    }
    // Authentication successful, proceed with plan calculation
    let body = req.into_body();
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from("Failed to read request body")))
                .unwrap());
        }
    };

    // Deserialize the request body into a PlanRequest
    let plan_request: PlanRequest = match serde_json::from_slice(&bytes) {
        Ok(plan_request) => plan_request,
        Err(e) => {
            error!("Failed to deserialize plan request: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from(format!(
                    "Invalid request format: {e}"
                ))))
                .unwrap());
        }
    };

    let current_infra_map = match InfrastructureMap::load_from_redis(redis_client).await {
        Ok(Some(infra_map)) => infra_map,
        Ok(None) => InfrastructureMap::default(),
        Err(e) => {
            error!("Failed to retrieve infrastructure map from Redis: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from(
                    "Failed to acquire lock on Redis client",
                )))
                .unwrap());
        }
    };

    // Calculate the changes between the submitted infrastructure map and the current one
    let changes = current_infra_map.diff(&plan_request.infra_map);

    // Prepare the response
    let response = PlanResponse {
        status: "success".to_string(),
        changes,
    };

    // Serialize the response to JSON
    let json_response = match serde_json::to_string(&response) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize plan response: {}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Full::new(Bytes::from("Internal server error")))
                .unwrap());
        }
    };

    // Return the response
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json_response)))
        .unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::framework::core::infrastructure::table::{Column, ColumnType, IntType, Table};
    use crate::framework::core::infrastructure_map::{
        OlapChange, PrimitiveSignature, PrimitiveTypes, TableChange,
    };
    use crate::framework::versions::Version;

    fn create_test_table(name: &str) -> Table {
        Table {
            name: name.to_string(),
            columns: vec![Column {
                name: "id".to_string(),
                data_type: ColumnType::Int(IntType::Int64),
                required: true,
                unique: true,
                primary_key: true,
                default: None,
                annotations: vec![],
            }],
            order_by: vec!["id".to_string()],
            engine: None,
            deduplicate: false,
            version: Some(Version::from_string("1.0.0".to_string())),
            source_primitive: PrimitiveSignature {
                name: "test".to_string(),
                primitive_type: PrimitiveTypes::DataModel,
            },
            metadata: None,
        }
    }

    fn create_test_infra_map() -> InfrastructureMap {
        InfrastructureMap::default()
    }

    #[tokio::test]
    async fn test_find_table_definition() {
        let table = create_test_table("test_table");
        let discrepancies = InfraDiscrepancies {
            unmapped_tables: vec![table.clone()],
            missing_tables: vec![],
            mismatched_tables: vec![OlapChange::Table(TableChange::Added(table.clone()))],
        };

        let result = find_table_definition("test_table", &discrepancies);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test_table");
    }

    #[tokio::test]
    async fn test_update_inframap_tables() {
        let table_name = "test_table";
        let test_table = create_test_table(table_name);

        let discrepancies = InfraDiscrepancies {
            unmapped_tables: vec![test_table.clone()],
            missing_tables: vec![],
            mismatched_tables: vec![OlapChange::Table(TableChange::Added(test_table.clone()))],
        };

        let mut infra_map = create_test_infra_map();

        let tables_to_update = vec![table_name.to_string()];
        let updated_tables =
            update_inframap_tables(tables_to_update, &discrepancies, &mut infra_map).await;

        assert_eq!(updated_tables.len(), 1);
        assert_eq!(updated_tables[0], table_name);
        assert!(infra_map.tables.contains_key(&test_table.id()));
        assert_eq!(
            infra_map.tables.get(&test_table.id()).unwrap().name,
            table_name
        );
    }

    #[tokio::test]
    async fn test_update_inframap_tables_unmapped() {
        let table_name = "unmapped_table";
        let test_table = create_test_table(table_name);

        let discrepancies = InfraDiscrepancies {
            unmapped_tables: vec![test_table.clone()],
            missing_tables: vec![],
            mismatched_tables: vec![OlapChange::Table(TableChange::Added(test_table.clone()))],
        };

        let mut infra_map = create_test_infra_map();

        let tables_to_update = vec![table_name.to_string()];
        let updated_tables =
            update_inframap_tables(tables_to_update, &discrepancies, &mut infra_map).await;

        assert_eq!(updated_tables.len(), 1);
        assert_eq!(updated_tables[0], table_name);
        assert!(infra_map.tables.contains_key(&test_table.id()));
        assert_eq!(
            infra_map.tables.get(&test_table.id()).unwrap().name,
            table_name
        );
    }
}
