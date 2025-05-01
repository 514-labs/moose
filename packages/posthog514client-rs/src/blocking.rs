//! Blocking (synchronous) PostHog client implementation

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use reqwest::blocking::Client as ReqwestClient;
use serde_json::json;

use crate::error::{ConfigErrorKind, PostHogError};
use crate::event::{Event514, MooseEventType};

const POSTHOG_HOST: &str = "https://us.i.posthog.com";
const POSTHOG_API_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");

/// Blocking (synchronous) client for PostHog analytics
#[derive(Debug, Clone)]
pub struct BlockingPostHog514Client {
    api_key: String,
    client: Arc<ReqwestClient>,
    host: String,
    machine_id: String,
}

impl BlockingPostHog514Client {
    /// Creates a new BlockingPostHog514Client with the given API key and machine ID
    pub fn new(
        api_key: impl Into<String>,
        machine_id: impl Into<String>,
    ) -> Result<Self, PostHogError> {
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
            machine_id: machine_id.into(),
        })
    }

    /// Creates a new BlockingPostHog514Client using the API key from the environment.
    /// This will:
    /// 1. First check for a build-time API key (baked into the binary)
    /// 2. Then check for a runtime POSTHOG_API_KEY environment variable
    /// 3. Return None if neither is available
    pub fn from_env(machine_id: impl Into<String>) -> Option<Self> {
        // First try build-time API key
        if let Some(api_key) = POSTHOG_API_KEY {
            return Self::new(api_key, machine_id).ok();
        }

        // Then try runtime environment variable
        if let Ok(api_key) = env::var("POSTHOG_API_KEY") {
            return Self::new(api_key, machine_id).ok();
        }

        None
    }

    /// Captures a custom event
    pub fn capture_event(
        &self,
        event_name: impl Into<String>,
        properties: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<(), PostHogError> {
        let event = Event514::new(event_name).with_distinct_id(self.machine_id.clone());

        if let Some(properties) = properties {
            let event = event.with_properties(properties);
            self.capture(event)
        } else {
            self.capture(event)
        }
    }

    pub fn capture_cli_command(
        &self,
        command: impl Into<String>,
        project: Option<String>,
        mut context: Option<HashMap<String, serde_json::Value>>,
        app_version: impl Into<String>,
        is_developer: bool,
    ) -> Result<(), PostHogError> {
        let mut event = Event514::new_moose(MooseEventType::MooseCliCommand)
            .with_distinct_id(self.machine_id.clone())
            .with_project(project);

        // Set 514-specific properties
        event.set_app_version(app_version);
        event.set_is_developer(is_developer);
        event.set_environment("production".to_string()); // TODO: Make configurable

        // Ensure we have a context to work with
        let context = context.get_or_insert_with(HashMap::new);

        // Add command to context if not already present
        if !context.contains_key("command") {
            context.insert("command".to_string(), json!(command.into()));
        }

        // Add context to event properties and capture
        let event = event.with_properties(context.clone());
        self.capture(event)
    }

    pub fn capture_cli_error(
        &self,
        error: impl std::error::Error,
        project: Option<String>,
        context: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<(), PostHogError> {
        let event = Event514::new_moose(MooseEventType::MooseCliError)
            .with_distinct_id(self.machine_id.clone())
            .with_project(project);

        if let Some(context) = context {
            let event = event.with_properties(context);
            let event = event.with_error(error);
            self.capture(event)
        } else {
            let event = event.with_error(error);
            self.capture(event)
        }
    }

    fn capture(&self, event: Event514) -> Result<(), PostHogError> {
        event.validate()?;

        let url = format!("{}/capture/", self.host);

        let payload = json!({
            "api_key": self.api_key,
            "event": event.event,
            "properties": event.properties,
            "distinct_id": event.distinct_id,
            "timestamp": event.timestamp,
        });

        let response = self.client.post(&url).json(&payload).send().map_err(|e| {
            PostHogError::send_event(
                "Failed to send request",
                Some(crate::error::SendEventErrorKind::Network(e.to_string())),
            )
        })?;

        match response.status() {
            status if status.is_success() => Ok(()),
            status if status.as_u16() == 429 => Err(PostHogError::send_event(
                "Rate limited",
                Some(crate::error::SendEventErrorKind::RateLimited),
            )),
            status if status.as_u16() == 401 => Err(PostHogError::send_event(
                "Authentication failed",
                Some(crate::error::SendEventErrorKind::Authentication),
            )),
            status => {
                let error_msg = response.text().unwrap_or_else(|_| "Unknown error".into());
                Err(PostHogError::send_event(
                    format!("Unexpected response: {}", status),
                    Some(crate::error::SendEventErrorKind::Network(error_msg)),
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[test]
    fn test_capture_cli_command() {
        let mut server = Server::new();
        let mock = server.mock("POST", "/capture/").with_status(200).create();

        let client = BlockingPostHog514Client::new("test_key", "test_machine_id").unwrap();
        let result = client.capture_cli_command(
            "test_command",
            Some("test_project".to_string()),
            None,
            "1.0.0",
            false,
        );

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_capture_cli_error() {
        let mut server = Server::new();
        let mock = server.mock("POST", "/capture/").with_status(200).create();

        let client = BlockingPostHog514Client::new("test_key", "test_machine_id").unwrap();
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let result = client.capture_cli_error(error, Some("test_project".to_string()), None);

        assert!(result.is_ok());
        mock.assert();
    }
}
