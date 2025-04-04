use chrono::{DateTime, Utc};
use serde_json::json;

use crate::cli::settings::Settings;
use crate::utilities::machine_id::get_or_create_machine_id;

// Build-time environment variable for PostHog API key
const POSTHOG_API_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");
const POSTHOG_HOST: &str = "https://us.i.posthog.com";

#[derive(Debug, thiserror::Error)]
pub enum PostHogError {
    #[error("Failed to send event to PostHog: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("PostHog API key not configured")]
    ApiKeyMissing,
}

#[derive(Debug, Clone)]
pub struct PostHogEvent {
    pub event: String,
    pub distinct_id: String,
    pub timestamp: DateTime<Utc>,
    pub properties: serde_json::Value,
}

#[derive(Clone)]
pub struct PostHogClient {
    api_key: Option<String>,
    host: String,
    machine_id: String,
    is_moose_developer: bool,
    http_client: reqwest::Client,
}

impl PostHogClient {
    pub fn new(settings: &Settings) -> Self {
        Self {
            api_key: POSTHOG_API_KEY.map(String::from),
            host: POSTHOG_HOST.to_string(),
            machine_id: get_or_create_machine_id(),
            is_moose_developer: settings.telemetry.is_moose_developer,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn capture(&self, event: PostHogEvent) -> Result<(), PostHogError> {
        // Skip telemetry if API key is not configured
        let api_key = match &self.api_key {
            Some(key) => key,
            None => return Ok(()),
        };

        let payload = json!({
            "api_key": api_key,
            "event": event.event,
            "distinct_id": event.distinct_id,
            "timestamp": event.timestamp,
            "properties": event.properties,
        });

        self.http_client
            .post(&format!("{}/capture/", self.host))
            .json(&payload)
            .send()
            .await
            .map_err(PostHogError::RequestError)?;

        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.api_key.is_some()
    }

    pub async fn capture_cli_usage(
        &self,
        event_name: &str,
        project_name: Option<String>,
        properties: serde_json::Value,
    ) -> Result<(), PostHogError> {
        let mut event_properties = json!({
            "cli_version": crate::utilities::constants::CLI_VERSION,
            "is_moose_developer": self.is_moose_developer,
            "project": project_name.unwrap_or_else(|| "N/A".to_string()),
        });

        // Merge additional properties
        if let Some(obj) = event_properties.as_object_mut() {
            if let Some(props) = properties.as_object() {
                obj.extend(props.clone());
            }
        }

        self.capture(PostHogEvent {
            event: event_name.to_string(),
            distinct_id: self.machine_id.clone(),
            timestamp: Utc::now(),
            properties: event_properties,
        })
        .await
    }
}
