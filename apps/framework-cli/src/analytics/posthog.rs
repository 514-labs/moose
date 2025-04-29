use chrono::{DateTime, Utc};
use posthog514client_rs::{Event, PostHogClient as BasePostHogClient, PostHogError};
use serde_json::json;

use crate::cli::settings::Settings;

// Build-time environment variable for PostHog API key
const POSTHOG_API_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");
const POSTHOG_HOST: &str = "https://us.i.posthog.com";

#[derive(Clone)]
pub struct PostHogClient {
    inner: Option<BasePostHogClient>,
    machine_id: String,
    is_moose_developer: bool,
}

impl PostHogClient {
    pub fn new(settings: &Settings, machine_id: String) -> Self {
        let inner =
            POSTHOG_API_KEY.and_then(|api_key| BasePostHogClient::new(api_key, POSTHOG_HOST).ok());

        Self {
            inner,
            machine_id,
            is_moose_developer: settings.telemetry.is_moose_developer,
        }
    }

    pub async fn capture(&self, event: PostHogEvent) -> Result<(), PostHogError> {
        // Skip telemetry if client is not configured
        let client = match &self.inner {
            Some(client) => client,
            None => return Ok(()),
        };

        let mut posthog_event = Event::new(event.event)
            .set_distinct_id(event.distinct_id)
            .set_timestamp(event.timestamp.to_rfc3339());

        // Add properties
        if let Some(props) = event.properties.as_object() {
            for (key, value) in props {
                posthog_event = posthog_event.add_property(key, value)?;
            }
        }

        client.capture(posthog_event).await
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.is_some()
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

// Keep the PostHogEvent struct for backward compatibility
#[derive(Debug, Clone)]
pub struct PostHogEvent {
    pub event: String,
    pub distinct_id: String,
    pub timestamp: DateTime<Utc>,
    pub properties: serde_json::Value,
}
