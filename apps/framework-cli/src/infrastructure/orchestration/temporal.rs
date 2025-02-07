use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use temporal_sdk_core_protos::temporal::api::workflowservice::v1::workflow_service_client::WorkflowServiceClient;
use tonic::transport::{Channel, Uri};

pub const DEFAULT_TEMPORTAL_NAMESPACE: &str = "default";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemporalConfig {
    #[serde(default = "default_db_user")]
    pub db_user: String,
    #[serde(default = "default_db_password")]
    pub db_password: String,
    #[serde(default = "default_db_port")]
    pub db_port: u16,
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
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            db_user: default_db_user(),
            db_password: default_db_password(),
            db_port: default_db_port(),
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

async fn connect_to_temporal() -> Result<WorkflowServiceClient<Channel>> {
    let endpoint: Uri = "http://localhost:7233".parse().unwrap();
    WorkflowServiceClient::connect(endpoint).await.map_err(|_| {
        Error::msg("Could not connect to Temporal. Please ensure the Temporal server is running.")
    })
}

pub async fn get_temporal_client() -> Result<WorkflowServiceClient<Channel>> {
    connect_to_temporal().await.map_err(|e| {
        eprintln!("{}", e);
        Error::msg(format!("{}", e))
    })
}
