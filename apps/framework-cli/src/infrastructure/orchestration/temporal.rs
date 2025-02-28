use anyhow::{Error, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use temporal_sdk_core_protos::temporal::api::workflowservice::v1::workflow_service_client::WorkflowServiceClient;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::{Channel, Uri};

use crate::framework::scripts::utils::{get_temporal_domain_name, get_temporal_namespace};

pub const DEFAULT_TEMPORTAL_NAMESPACE: &str = "default";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemporalConfig {
    #[serde(default = "default_db_user")]
    pub db_user: String,
    #[serde(default = "default_db_password")]
    pub db_password: String,
    #[serde(default = "default_db_port")]
    pub db_port: u16,
    #[serde(default = "default_temporal_host")]
    pub temporal_host: String,
    #[serde(default = "default_temporal_port")]
    pub temporal_port: u16,
    #[serde(default = "default_temporal_version")]
    pub temporal_version: String,
    #[serde(default = "default_admin_tools_version")]
    pub admin_tools_version: String,
    #[serde(default = "default_ui_version")]
    pub ui_version: String,
    #[serde(default = "default_ui_port")]
    pub ui_port: u16,
    #[serde(default = "default_ui_cors_origins")]
    pub ui_cors_origins: String,
    #[serde(default = "default_config_path")]
    pub config_path: String,
    #[serde(default = "default_postgresql_version")]
    pub postgresql_version: String,
}

fn default_db_user() -> String {
    "temporal".to_string()
}

fn default_db_password() -> String {
    "temporal".to_string()
}

fn default_db_port() -> u16 {
    5432
}

fn default_temporal_host() -> String {
    "localhost".to_string()
}

fn default_temporal_port() -> u16 {
    7233
}

fn default_temporal_version() -> String {
    "1.22.3".to_string()
}

fn default_admin_tools_version() -> String {
    "1.22.3".to_string()
}

fn default_ui_version() -> String {
    "2.21.3".to_string()
}

fn default_ui_port() -> u16 {
    8080
}

fn default_ui_cors_origins() -> String {
    "http://localhost:3000".to_string()
}

fn default_config_path() -> String {
    "config/dynamicconfig/development-sql.yaml".to_string()
}

fn default_postgresql_version() -> String {
    "13".to_string()
}

impl TemporalConfig {
    pub fn to_env_vars(&self) -> Vec<(String, String)> {
        vec![
            ("TEMPORAL_DB_USER".to_string(), self.db_user.clone()),
            ("TEMPORAL_DB_PASSWORD".to_string(), self.db_password.clone()),
            ("TEMPORAL_DB_PORT".to_string(), self.db_port.to_string()),
            ("TEMPORAL_PORT".to_string(), self.temporal_port.to_string()),
            (
                "TEMPORAL_VERSION".to_string(),
                self.temporal_version.clone(),
            ),
            (
                "TEMPORAL_ADMINTOOLS_VERSION".to_string(),
                self.admin_tools_version.clone(),
            ),
            ("TEMPORAL_UI_VERSION".to_string(), self.ui_version.clone()),
            ("TEMPORAL_UI_PORT".to_string(), self.ui_port.to_string()),
            (
                "TEMPORAL_UI_CORS_ORIGINS".to_string(),
                self.ui_cors_origins.clone(),
            ),
            ("TEMPORAL_CONFIG_PATH".to_string(), self.config_path.clone()),
            (
                "POSTGRESQL_VERSION".to_string(),
                self.postgresql_version.clone(),
            ),
        ]
    }

    /// Temporal TS/PY sdk expects a url without a scheme
    pub fn temporal_url(&self) -> String {
        format!("{}:{}", self.temporal_host, self.temporal_port)
    }

    /// Temporal Rust sdk expects a scheme for the temporal url
    pub fn temporal_url_with_scheme(&self) -> String {
        let scheme = if self.temporal_host == "localhost" {
            "http"
        } else {
            "https"
        };
        format!("{}://{}:{}", scheme, self.temporal_host, self.temporal_port)
    }
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            db_user: default_db_user(),
            db_password: default_db_password(),
            db_port: default_db_port(),
            temporal_host: default_temporal_host(),
            temporal_port: default_temporal_port(),
            temporal_version: default_temporal_version(),
            admin_tools_version: default_admin_tools_version(),
            ui_version: default_ui_version(),
            ui_port: default_ui_port(),
            ui_cors_origins: default_ui_cors_origins(),
            config_path: default_config_path(),
            postgresql_version: default_postgresql_version(),
        }
    }
}

lazy_static! {
    pub static ref MOOSE_TEMPORAL_CONFIG__CA_CERT: String =
        get_env_var("MOOSE_TEMPORAL_CONFIG__CA_CERT");
    pub static ref MOOSE_TEMPORAL_CONFIG__CLIENT_CERT: String =
        get_env_var("MOOSE_TEMPORAL_CONFIG__CLIENT_CERT");
    pub static ref MOOSE_TEMPORAL_CONFIG__CLIENT_KEY: String =
        get_env_var("MOOSE_TEMPORAL_CONFIG__CLIENT_KEY");
    pub static ref MOOSE_TEMPORAL_CONFIG__API_KEY: String =
        get_env_var("MOOSE_TEMPORAL_CONFIG__API_KEY");
}

pub struct ApiKeyInterceptor {
    api_key: String,
    namespace: String,
}

impl tonic::service::Interceptor for ApiKeyInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        request.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::try_from(format!("Bearer {}", self.api_key))
                .map_err(|_| tonic::Status::internal("Invalid metadata value"))?,
        );
        request.metadata_mut().insert(
            "temporal-namespace",
            tonic::metadata::MetadataValue::try_from(&self.namespace)
                .map_err(|_| tonic::Status::internal("Invalid metadata value"))?,
        );
        Ok(request)
    }
}

async fn connect_to_temporal(temporal_url: &str) -> Result<WorkflowServiceClient<Channel>> {
    let endpoint: Uri = temporal_url.parse().unwrap();
    WorkflowServiceClient::connect(endpoint).await.map_err(|_| {
        Error::msg("Could not connect to Temporal. Please ensure the Temporal server is running.")
    })
}

pub async fn get_temporal_client(temporal_url: &str) -> Result<WorkflowServiceClient<Channel>> {
    connect_to_temporal(temporal_url).await.map_err(|e| {
        eprintln!("{}", e);
        Error::msg(format!("{}", e))
    })
}

pub async fn get_temporal_client_mtls(
    temporal_url: &str,
) -> Result<WorkflowServiceClient<Channel>> {
    let ca_cert_path = MOOSE_TEMPORAL_CONFIG__CA_CERT.clone();
    let client_cert_path = MOOSE_TEMPORAL_CONFIG__CLIENT_CERT.clone();
    let client_key_path = MOOSE_TEMPORAL_CONFIG__CLIENT_KEY.clone();

    let domain_name = get_temporal_domain_name(temporal_url);

    let client_identity = tonic::transport::Identity::from_pem(
        std::fs::read(client_cert_path).map_err(|e| Error::msg(e.to_string()))?,
        std::fs::read(client_key_path).map_err(|e| Error::msg(e.to_string()))?,
    );

    let ca_certificate = tonic::transport::Certificate::from_pem(
        std::fs::read(ca_cert_path).map_err(|e| Error::msg(e.to_string()))?,
    );

    let tls_config = tonic::transport::ClientTlsConfig::new()
        .identity(client_identity)
        .ca_certificate(ca_certificate)
        .domain_name(domain_name);

    let endpoint = tonic::transport::Channel::from_shared(temporal_url.to_string())
        .map_err(|e| Error::msg(e.to_string()))?;

    let client = WorkflowServiceClient::new(endpoint.tls_config(tls_config)?.connect().await?);

    Ok(client)
}

pub async fn get_temporal_client_api_key(
    temporal_url: &str,
) -> Result<WorkflowServiceClient<InterceptedService<Channel, ApiKeyInterceptor>>> {
    let ca_cert_path = MOOSE_TEMPORAL_CONFIG__CA_CERT.clone();
    let api_key = MOOSE_TEMPORAL_CONFIG__API_KEY.clone();

    let domain_name = get_temporal_domain_name(temporal_url);

    let namespace = get_temporal_namespace(domain_name);

    let ca_certificate = tonic::transport::Certificate::from_pem(
        std::fs::read(ca_cert_path).map_err(|e| Error::msg(e.to_string()))?,
    );

    let endpoint =
        tonic::transport::Channel::from_shared("https://us-west1.gcp.api.temporal.io:7233")
            .map_err(|e| Error::msg(e.to_string()))?
            .tls_config(
                tonic::transport::ClientTlsConfig::new()
                    .domain_name("us-west1.gcp.api.temporal.io")
                    .ca_certificate(ca_certificate),
            )
            .map_err(|e| Error::msg(e.to_string()))?;

    let interceptor = ApiKeyInterceptor {
        api_key: api_key.to_string(),
        namespace: namespace.clone(),
    };

    let client = WorkflowServiceClient::with_interceptor(endpoint.connect_lazy(), interceptor);

    Ok(client)
}

fn get_env_var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| "".to_string())
}
