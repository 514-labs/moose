//! PostHog client library for Rust
//!
//! This crate provides a client for interacting with PostHog analytics.

pub mod client;
pub mod error;
pub mod event;

pub use client::{Config, PostHog514Client, PostHogClient};
pub use error::{ConfigErrorKind, PostHogError, SendEventErrorKind};
pub use event::{Event514, EventType};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_event_capture() {
        let client = PostHogClient::new("test_key", "https://app.posthog.com").unwrap();
        let event = Event514::new(EventType::MooseCliCommand)
            .with_distinct_id("test_user")
            .with_properties([("test".to_string(), "value".into())].into());

        assert!(client.capture(event).await.is_err());
    }
}
