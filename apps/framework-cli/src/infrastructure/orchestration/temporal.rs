use serde::{Deserialize, Serialize};

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
    #[serde(default = "default_client_cert")]
    pub client_cert: String,
    #[serde(default = "default_client_key")]
    pub client_key: String,
    #[serde(default = "default_ca_cert")]
    pub ca_cert: String,
    #[serde(default = "default_api_key")]
    pub api_key: String,
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

fn default_client_cert() -> String {
    "".to_string()
}

fn default_client_key() -> String {
    "".to_string()
}

fn default_ca_cert() -> String {
    "".to_string()
}

fn default_api_key() -> String {
    "".to_string()
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
            client_cert: default_client_cert(),
            client_key: default_client_key(),
            ca_cert: default_ca_cert(),
            api_key: default_api_key(),
        }
    }
}
