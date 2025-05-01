//! Types for PostHog events and properties

use crate::error::{PostHogError, SerializationErrorKind};
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;

/// Event types supported by the PostHog client
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum EventType {
    /// Built-in event types for Moose CLI
    #[serde(rename_all = "snake_case")]
    Moose(MooseEventType),
    /// Custom event type that can be any string
    Custom(String),
}

/// Built-in event types for Moose CLI
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MooseEventType {
    MooseCliCommand,
    MooseCliError,
}

#[derive(Debug, Clone, Serialize)]
pub struct Event514 {
    #[serde(flatten)]
    pub event: EventType,
    pub distinct_id: String,
    #[serde(flatten)]
    pub properties: Properties514,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Properties514 {
    // Common properties across all 514 apps
    pub app_version: String,
    pub is_developer: bool,
    pub environment: String,
    pub project: Option<String>,
    // Event-specific properties
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Event514 {
    /// Creates a new event with a built-in Moose event type
    pub fn new_moose(event_type: MooseEventType) -> Self {
        Self {
            event: EventType::Moose(event_type),
            distinct_id: String::new(),
            properties: Properties514::default(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    /// Creates a new event with a custom event type
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event: EventType::Custom(event_type.into()),
            distinct_id: String::new(),
            properties: Properties514::default(),
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    pub fn with_distinct_id(mut self, id: impl Into<String>) -> Self {
        self.distinct_id = id.into();
        self
    }

    pub fn with_project(mut self, project: Option<String>) -> Self {
        self.properties.project = project;
        self
    }

    pub fn with_properties(mut self, properties: HashMap<String, serde_json::Value>) -> Self {
        self.properties.custom = properties;
        self
    }

    pub fn with_error(mut self, error: impl std::error::Error) -> Self {
        self.properties.custom.insert(
            "error".to_string(),
            serde_json::Value::String(error.to_string()),
        );
        self.properties.custom.insert(
            "error_type".to_string(),
            serde_json::Value::String(std::any::type_name_of_val(&error).to_string()),
        );
        self
    }

    pub fn with_context(mut self, context: HashMap<String, serde_json::Value>) -> Self {
        self.properties.custom.extend(context);
        self
    }

    pub fn set_app_version(&mut self, version: impl Into<String>) {
        self.properties.app_version = version.into();
    }

    pub fn set_is_developer(&mut self, is_developer: bool) {
        self.properties.is_developer = is_developer;
    }

    pub fn set_environment(&mut self, env: impl Into<String>) {
        self.properties.environment = env.into();
    }
}

impl Event514 {
    /// Validates that the event has all required fields
    pub(crate) fn validate(&self) -> Result<(), PostHogError> {
        if self.distinct_id.is_empty() {
            return Err(PostHogError::serialization(
                "distinct_id cannot be empty",
                Some(SerializationErrorKind::InvalidPropertyValue(
                    "empty distinct_id".into(),
                )),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moose_event_creation() {
        let event = Event514::new_moose(MooseEventType::MooseCliCommand)
            .with_distinct_id("test-id")
            .with_project(Some("test-project".to_string()));

        assert!(matches!(
            event.event,
            EventType::Moose(MooseEventType::MooseCliCommand)
        ));
        assert_eq!(event.distinct_id, "test-id");
        assert_eq!(event.properties.project, Some("test-project".to_string()));
    }

    #[test]
    fn test_custom_event_creation() {
        let event = Event514::new("user.signup")
            .with_distinct_id("test-id")
            .with_project(Some("test-project".to_string()));

        assert!(matches!(event.event, EventType::Custom(ref name) if name == "user.signup"));
        assert_eq!(event.distinct_id, "test-id");
        assert_eq!(event.properties.project, Some("test-project".to_string()));
    }

    #[test]
    fn test_error_event() {
        let error = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
        let event = Event514::new_moose(MooseEventType::MooseCliError)
            .with_distinct_id("test-id")
            .with_error(error);

        assert!(matches!(
            event.event,
            EventType::Moose(MooseEventType::MooseCliError)
        ));
        assert!(event.properties.custom.contains_key("error"));
        assert!(event.properties.custom.contains_key("error_type"));
    }

    #[test]
    fn test_event_validation() {
        let event = Event514::new("custom.event");
        assert!(event.validate().is_err());

        let event = Event514::new("custom.event").with_distinct_id("user123");
        assert!(event.validate().is_ok());
    }
}
