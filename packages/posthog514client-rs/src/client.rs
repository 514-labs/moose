//! PostHog client implementation

use reqwest::{Client as ReqwestClient, Url};
use serde_json::json;
use std::time::Duration;
use tracing::{debug, error, instrument};

use crate::error::{ConfigErrorKind, PostHogError, SendEventErrorKind};
use crate::event::Event;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

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
    #[instrument(skip(self, event), fields(event_name = %event.event))]
    pub async fn capture(&self, event: Event) -> Result<(), PostHogError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
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
                "event": "test_event",
                "distinct_id": "user123",
                "properties": {"test": "value"}
            })))
            .with_status(200)
            .create();

        let client = PostHogClient::new("test_key", server.url()).unwrap();
        let event = Event::new("test_event")
            .set_distinct_id("user123")
            .add_property("test", "value")
            .unwrap();

        assert!(client.capture(event).await.is_ok());
        mock.assert();
    }
}
