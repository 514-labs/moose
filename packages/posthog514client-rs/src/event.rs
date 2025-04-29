//! Types for PostHog events and properties

use crate::error::{PostHogError, SerializationErrorKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a PostHog event with its properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// The name of the event
    pub event: String,

    /// Unique identifier for the user/entity this event is associated with
    #[serde(rename = "distinct_id")]
    pub distinct_id: String,

    /// Custom properties associated with the event
    pub properties: HashMap<String, serde_json::Value>,

    /// Timestamp when the event occurred (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl Event {
    /// Creates a new event with the given name
    pub fn new(event: impl Into<String>) -> Self {
        Self {
            event: event.into(),
            distinct_id: String::new(),
            properties: HashMap::new(),
            timestamp: None,
        }
    }

    /// Sets the distinct_id for this event
    pub fn set_distinct_id(mut self, distinct_id: impl Into<String>) -> Self {
        self.distinct_id = distinct_id.into();
        self
    }

    /// Adds a property to the event
    pub fn add_property<K, V>(mut self, key: K, value: V) -> Result<Self, PostHogError>
    where
        K: Into<String>,
        V: Serialize,
    {
        let value = serde_json::to_value(value).map_err(|e| {
            PostHogError::serialization(
                "Failed to serialize property value",
                Some(SerializationErrorKind::Json(e.to_string())),
            )
        })?;

        self.properties.insert(key.into(), value);
        Ok(self)
    }

    /// Sets the timestamp for this event
    pub fn set_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Validates that the event has all required fields
    pub(crate) fn validate(&self) -> Result<(), PostHogError> {
        if self.event.is_empty() {
            return Err(PostHogError::serialization(
                "Event name cannot be empty",
                Some(SerializationErrorKind::InvalidPropertyValue(
                    "empty event name".into(),
                )),
            ));
        }

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
    use serde_json::json;

    #[test]
    fn test_event_creation_and_properties() {
        let event = Event::new("test_event")
            .set_distinct_id("user123")
            .add_property("key1", "value1")
            .unwrap()
            .add_property("key2", 42)
            .unwrap();

        assert_eq!(event.event, "test_event");
        assert_eq!(event.distinct_id, "user123");
        assert_eq!(event.properties.get("key1").unwrap(), &json!("value1"));
        assert_eq!(event.properties.get("key2").unwrap(), &json!(42));
    }

    #[test]
    fn test_event_validation() {
        let event = Event::new("");
        assert!(event.validate().is_err());

        let event = Event::new("test_event");
        assert!(event.validate().is_err());

        let event = Event::new("test_event").set_distinct_id("user123");
        assert!(event.validate().is_ok());
    }
}
