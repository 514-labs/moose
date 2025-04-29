//! PostHog client implementation

use reqwest::{Client as ReqwestClient, Url};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, instrument};

use crate::error::{ConfigErrorKind, PostHogError, SendEventErrorKind};
use crate::event::{Event514, EventType};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const POSTHOG_HOST: &str = "https://app.posthog.com";

/// Configuration for the PostHog client
#[derive(Debug, Clone)]
pub struct Config {
    /// API key for authentication
    api_key: String,
    /// Base URL for the PostHog instance
    base_url: Url,
    /// Request timeout
    timeout: Duration,
}

impl Config {
    /// Creates a new configuration with custom settings
    pub fn new(
        api_key: impl Into<String>,
        base_url: impl AsRef<str>,
    ) -> Result<Self, PostHogError> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(PostHogError::configuration(
                "API key cannot be empty",
                Some(ConfigErrorKind::InvalidApiKey),
            ));
        }

        let base_url = Url::parse(base_url.as_ref()).map_err(|e| {
            PostHogError::configuration(
                "Invalid base URL",
                Some(ConfigErrorKind::InvalidUrl(e.to_string())),
            )
        })?;

        Ok(Self {
            api_key,
            base_url,
            timeout: DEFAULT_TIMEOUT,
        })
    }

    /// Sets a custom timeout for requests
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Client for interacting with the PostHog API
#[derive(Debug, Clone)]
pub struct PostHogClient {
    config: Config,
    client: ReqwestClient,
}

impl PostHogClient {
    /// Creates a new PostHog client with default configuration
    pub fn new(
        api_key: impl Into<String>,
        base_url: impl AsRef<str>,
    ) -> Result<Self, PostHogError> {
        let config = Config::new(api_key, base_url)?;
        let client = ReqwestClient::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| {
                PostHogError::configuration(
                    "Failed to create HTTP client",
                    Some(ConfigErrorKind::InvalidUrl(e.to_string())),
                )
            })?;

        Ok(Self { config, client })
    }

    /// Creates a new PostHog client with custom configuration
    pub fn with_config(config: Config) -> Result<Self, PostHogError> {
        let client = ReqwestClient::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| {
                PostHogError::configuration(
                    "Failed to create HTTP client",
                    Some(ConfigErrorKind::InvalidUrl(e.to_string())),
                )
            })?;

        Ok(Self { config, client })
    }

    /// Captures a single event
    #[instrument(skip(self, event), fields(event_name = ?event.event))]
    pub async fn capture(&self, event: Event514) -> Result<(), PostHogError> {
        event.validate()?;

        let url = self.config.base_url.join("/capture/").map_err(|e| {
            PostHogError::configuration(
                "Failed to construct capture URL",
                Some(ConfigErrorKind::InvalidUrl(e.to_string())),
            )
        })?;

        let payload = json!({
            "api_key": self.config.api_key,
            "event": event.event,
            "properties": event.properties,
            "distinct_id": event.distinct_id,
            "timestamp": event.timestamp,
        });

        debug!("Sending event to PostHog");

        let response = self
            .client
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                PostHogError::send_event(
                    "Failed to send request",
                    Some(SendEventErrorKind::Network(e.to_string())),
                )
            })?;

        match response.status() {
            status if status.is_success() => {
                debug!("Successfully sent event to PostHog");
                Ok(())
            }
            status if status.as_u16() == 429 => {
                error!("Rate limited by PostHog API");
                Err(PostHogError::send_event(
                    "Rate limited",
                    Some(SendEventErrorKind::RateLimited),
                ))
            }
            status if status.as_u16() == 401 => {
                error!("Authentication failed");
                Err(PostHogError::send_event(
                    "Authentication failed",
                    Some(SendEventErrorKind::Authentication),
                ))
            }
            status => {
                let error_msg = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".into());
                error!("Unexpected response from PostHog: {}", error_msg);
                Err(PostHogError::send_event(
                    format!("Unexpected response: {}", status),
                    Some(SendEventErrorKind::Network(error_msg)),
                ))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PostHog514Client {
    api_key: String,
    client: Arc<ReqwestClient>,
    host: String,
}

impl PostHog514Client {
    pub fn new(api_key: impl Into<String>) -> Result<Self, PostHogError> {
        let client = ReqwestClient::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| {
                PostHogError::configuration(
                    "Failed to create HTTP client",
                    Some(ConfigErrorKind::InvalidUrl(e.to_string())),
                )
            })?;

        Ok(Self {
            api_key: api_key.into(),
            client: Arc::new(client),
            host: POSTHOG_HOST.to_string(),
        })
    }

    pub async fn capture_cli_command(
        &self,
        distinct_id: impl Into<String>,
        command: impl Into<String>,
        project: Option<String>,
        context: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<(), PostHogError> {
        let mut event = Event514::new(EventType::MooseCliCommand)
            .with_distinct_id(distinct_id)
            .with_project(project);

        if let Some(ctx) = context {
            event = event.with_context(ctx);
        }

        event.properties.custom.insert(
            "command".to_string(),
            serde_json::Value::String(command.into()),
        );

        self.capture(event).await
    }

    pub async fn capture_cli_error(
        &self,
        distinct_id: impl Into<String>,
        error: impl std::error::Error,
        project: Option<String>,
        context: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<(), PostHogError> {
        let mut event = Event514::new(EventType::MooseCliError)
            .with_distinct_id(distinct_id)
            .with_project(project)
            .with_error(error);

        if let Some(ctx) = context {
            event = event.with_context(ctx);
        }

        self.capture(event).await
    }

    async fn capture(&self, event: Event514) -> Result<(), PostHogError> {
        event.validate()?;

        let url = format!("{}/capture/", self.host);
        let payload = json!({
            "api_key": self.api_key,
            "event": event.event,
            "properties": event.properties,
            "timestamp": event.timestamp,
            "distinct_id": event.distinct_id,
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                PostHogError::send_event(
                    "Failed to send request",
                    Some(SendEventErrorKind::Network(e.to_string())),
                )
            })?;

        match response.status() {
            reqwest::StatusCode::OK | reqwest::StatusCode::ACCEPTED => Ok(()),
            reqwest::StatusCode::UNAUTHORIZED => Err(PostHogError::send_event(
                "Invalid API key",
                Some(SendEventErrorKind::Authentication),
            )),
            reqwest::StatusCode::TOO_MANY_REQUESTS => Err(PostHogError::send_event(
                "Too many requests",
                Some(SendEventErrorKind::RateLimited),
            )),
            status => Err(PostHogError::send_event(
                "Unexpected response from PostHog",
                Some(SendEventErrorKind::Network(status.to_string())),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use std::io;
    use test_case::test_case;

    #[test_case("", "https://app.posthog.com" => Err(()) ; "empty api key")]
    #[test_case("test_key", "invalid-url" => Err(()) ; "invalid url")]
    #[test_case("test_key", "https://app.posthog.com" => Ok(()) ; "valid config")]
    fn test_config_creation(api_key: &str, base_url: &str) -> Result<(), ()> {
        match Config::new(api_key, base_url) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    #[tokio::test]
    async fn test_capture_success() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/capture/")
            .match_body(mockito::Matcher::Json(json!({
                "api_key": "test_key",
                "event": "moose_cli_command",
                "distinct_id": "user123",
                "properties": {}
            })))
            .with_status(200)
            .create();

        let client = PostHogClient::new("test_key", server.url()).unwrap();
        let event = Event514::new(EventType::MooseCliCommand).with_distinct_id("user123");

        assert!(client.capture(event).await.is_ok());
        mock.assert();
    }

    #[tokio::test]
    async fn test_capture_cli_command() {
        let client = PostHog514Client::new("test_key").unwrap();
        let result = client
            .capture_cli_command(
                "test-user",
                "moose init",
                Some("test-project".to_string()),
                None,
            )
            .await;

        // This will fail since we're using a fake API key
        assert!(matches!(
            result,
            Err(PostHogError::SendEvent {
                source: Some(SendEventErrorKind::Authentication),
                ..
            })
        ));
    }

    #[tokio::test]
    async fn test_capture_cli_error() {
        let client = PostHog514Client::new("test_key").unwrap();
        let error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let result = client
            .capture_cli_error("test-user", error, Some("test-project".to_string()), None)
            .await;

        // This will fail since we're using a fake API key
        assert!(matches!(
            result,
            Err(PostHogError::SendEvent {
                source: Some(SendEventErrorKind::Authentication),
                ..
            })
        ));
    }
}
