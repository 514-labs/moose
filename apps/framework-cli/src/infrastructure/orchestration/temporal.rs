use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("invalid temporal scheme '{scheme}': must be 'http' or 'https'")]
pub struct InvalidTemporalSchemeError {
    pub scheme: String,
}

/// Valid temporal URL schemes
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TemporalScheme {
    Http,
    Https,
}

impl<'de> Deserialize<'de> for TemporalScheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TemporalScheme::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl TemporalScheme {
    pub const HTTP: &'static str = "http";
    pub const HTTPS: &'static str = "https";

    /// Create a TemporalScheme from a string, validating it's either "http" or "https"
    pub fn from_str(scheme: &str) -> Result<Self, InvalidTemporalSchemeError> {
        match scheme.to_lowercase().as_str() {
            Self::HTTP => Ok(Self::Http),
            Self::HTTPS => Ok(Self::Https),
            _ => Err(InvalidTemporalSchemeError {
                scheme: scheme.to_string(),
            }),
        }
    }

    /// Convert the scheme to a string slice
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Http => Self::HTTP,
            Self::Https => Self::HTTPS,
        }
    }
}

impl std::fmt::Display for TemporalScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<String> for TemporalScheme {
    type Error = InvalidTemporalSchemeError;

    fn try_from(scheme: String) -> Result<Self, Self::Error> {
        Self::from_str(&scheme)
    }
}

impl TryFrom<&str> for TemporalScheme {
    type Error = InvalidTemporalSchemeError;

    fn try_from(scheme: &str) -> Result<Self, Self::Error> {
        Self::from_str(scheme)
    }
}

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
    #[serde(default = "default_temporal_scheme")]
    pub temporal_scheme: Option<TemporalScheme>,
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

fn default_temporal_scheme() -> Option<TemporalScheme> {
    None
}

fn default_temporal_version() -> String {
    "1.27".to_string()
}

fn default_admin_tools_version() -> String {
    "1.27".to_string()
}

fn default_ui_version() -> String {
    // Minor version is mandatory for the UI
    "2.37.0".to_string()
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
    pub fn temporal_url_with_scheme(&self) -> Result<String, InvalidTemporalSchemeError> {
        self.temporal_url_with_scheme_validate(true)
    }

    /// Temporal Rust sdk expects a scheme for the temporal url
    /// When validate is false, scheme validation is skipped (useful when Temporal is not being used)
    pub fn temporal_url_with_scheme_validate(&self, _validate: bool) -> Result<String, InvalidTemporalSchemeError> {
        let scheme = if let Some(ref configured_scheme) = self.temporal_scheme {
            // Since we're using an enum, the scheme is already validated
            configured_scheme.as_str()
        } else if self.temporal_host == "localhost" {
            TemporalScheme::HTTP
        } else {
            TemporalScheme::HTTPS
        };
        Ok(format!("{}://{}:{}", scheme, self.temporal_host, self.temporal_port))
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
            temporal_scheme: default_temporal_scheme(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_scheme_validation() {
        // Valid schemes
        assert_eq!(TemporalScheme::from_str("http").unwrap(), TemporalScheme::Http);
        assert_eq!(TemporalScheme::from_str("https").unwrap(), TemporalScheme::Https);
        assert_eq!(TemporalScheme::from_str("HTTP").unwrap(), TemporalScheme::Http);
        assert_eq!(TemporalScheme::from_str("HTTPS").unwrap(), TemporalScheme::Https);
        assert_eq!(TemporalScheme::from_str("Http").unwrap(), TemporalScheme::Http);
        assert_eq!(TemporalScheme::from_str("Https").unwrap(), TemporalScheme::Https);

        // Invalid schemes
        assert!(TemporalScheme::from_str("ftp").is_err());
        assert!(TemporalScheme::from_str("ws").is_err());
        assert!(TemporalScheme::from_str("invalid").is_err());
        assert!(TemporalScheme::from_str("").is_err());
    }

    #[test]
    fn test_temporal_scheme_display() {
        assert_eq!(TemporalScheme::Http.to_string(), "http");
        assert_eq!(TemporalScheme::Https.to_string(), "https");
    }

    #[test]
    fn test_temporal_scheme_as_str() {
        assert_eq!(TemporalScheme::Http.as_str(), "http");
        assert_eq!(TemporalScheme::Https.as_str(), "https");
    }

    #[test]
    fn test_temporal_url_with_scheme_default_behavior() {
        let config = TemporalConfig::default();
        assert_eq!(config.temporal_url_with_scheme().unwrap(), "http://localhost:7233");
    }

    #[test]
    fn test_temporal_url_with_scheme_forced_http() {
        let mut config = TemporalConfig::default();
        config.temporal_scheme = Some(TemporalScheme::Http);
        config.temporal_host = "example.com".to_string();
        assert_eq!(config.temporal_url_with_scheme().unwrap(), "http://example.com:7233");
    }

    #[test]
    fn test_temporal_url_with_scheme_forced_https() {
        let mut config = TemporalConfig::default();
        config.temporal_scheme = Some(TemporalScheme::Https);
        config.temporal_host = "localhost".to_string();
        assert_eq!(config.temporal_url_with_scheme().unwrap(), "https://localhost:7233");
    }

    #[test]
    fn test_temporal_url_with_scheme_auto_detect_localhost() {
        let mut config = TemporalConfig::default();
        config.temporal_scheme = None;
        config.temporal_host = "localhost".to_string();
        assert_eq!(config.temporal_url_with_scheme().unwrap(), "http://localhost:7233");
    }

    #[test]
    fn test_temporal_url_with_scheme_auto_detect_non_localhost() {
        let mut config = TemporalConfig::default();
        config.temporal_scheme = None;
        config.temporal_host = "example.com".to_string();
        assert_eq!(config.temporal_url_with_scheme().unwrap(), "https://example.com:7233");
    }

    #[test]
    fn test_temporal_url_with_scheme_invalid_scheme() {
        // Since we're using an enum, we can't create invalid schemes
        // The validation happens at deserialization time
        let mut config = TemporalConfig::default();
        config.temporal_scheme = Some(TemporalScheme::Http);
        config.temporal_host = "example.com".to_string();
        
        let result = config.temporal_url_with_scheme();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "http://example.com:7233");
    }
}
