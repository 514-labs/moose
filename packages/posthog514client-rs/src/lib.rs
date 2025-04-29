//! PostHog client library for Rust
//!
//! This crate provides a client for interacting with PostHog analytics.

mod client;
mod error;
mod event;

pub use client::{Config, PostHogClient};
pub use error::{ConfigErrorKind, PostHogError, SendEventErrorKind, SerializationErrorKind};
pub use event::Event;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_event_capture() {
        let client = PostHogClient::new("test_key", "https://app.posthog.com").unwrap();
        let event = Event::new("test_event")
            .set_distinct_id("test_user")
            .add_property("test", "value")
            .unwrap();

        // This will fail since we're not using a mock server
        assert!(client.capture(event).await.is_err());
    }
}
