//! Error types for the PostHog client

use std::fmt;
use thiserror::Error;

/// Represents errors that can occur when using the PostHog client
#[derive(Debug, Error, Clone)]
#[non_exhaustive]
pub enum PostHogError {
    /// Failed to send an event to PostHog
    #[error("failed to send event to PostHog: {message}")]
    SendEvent {
        message: String,
        #[source]
        source: Option<SendEventErrorKind>,
    },

    /// Failed to create or validate client configuration
    #[error("invalid client configuration: {message}")]
    Configuration {
        message: String,
        #[source]
        source: Option<ConfigErrorKind>,
    },

    /// Failed to serialize or deserialize data
    #[error("serialization error: {message}")]
    Serialization {
        message: String,
        #[source]
        source: Option<SerializationErrorKind>,
    },
}

/// Specific kinds of errors that can occur when sending events
#[derive(Debug, Error, Clone)]
#[non_exhaustive]
pub enum SendEventErrorKind {
    /// Network-related errors
    #[error("network error: {0}")]
    Network(String),

    /// Rate limiting errors
    #[error("rate limited by PostHog API")]
    RateLimited,

    /// Authentication errors
    #[error("authentication failed: invalid API key")]
    Authentication,
}

/// Configuration-related error kinds
#[derive(Debug, Error, Clone)]
#[non_exhaustive]
pub enum ConfigErrorKind {
    /// Invalid API key format
    #[error("invalid API key format")]
    InvalidApiKey,

    /// Invalid PostHog instance URL
    #[error("invalid PostHog URL: {0}")]
    InvalidUrl(String),
}

/// Serialization-related error kinds
#[derive(Debug, Error, Clone)]
#[non_exhaustive]
pub enum SerializationErrorKind {
    /// JSON serialization errors
    #[error("JSON error: {0}")]
    Json(String),

    /// Invalid event property value
    #[error("invalid property value: {0}")]
    InvalidPropertyValue(String),
}

impl PostHogError {
    /// Creates a new SendEvent error
    pub(crate) fn send_event<T: fmt::Display>(
        message: T,
        source: Option<SendEventErrorKind>,
    ) -> Self {
        Self::SendEvent {
            message: message.to_string(),
            source,
        }
    }

    /// Creates a new Configuration error
    pub(crate) fn configuration<T: fmt::Display>(
        message: T,
        source: Option<ConfigErrorKind>,
    ) -> Self {
        Self::Configuration {
            message: message.to_string(),
            source,
        }
    }

    /// Creates a new Serialization error
    pub(crate) fn serialization<T: fmt::Display>(
        message: T,
        source: Option<SerializationErrorKind>,
    ) -> Self {
        Self::Serialization {
            message: message.to_string(),
            source,
        }
    }
}
