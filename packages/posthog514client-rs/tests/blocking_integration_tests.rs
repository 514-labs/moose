#[cfg(feature = "blocking")]
mod tests {
    use mockito::Server;
    use posthog514client_rs::{BlockingPostHog514Client, PostHogError, SendEventErrorKind};
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_blocking_client_thread_safety() {
        let mut server = Server::new();
        let mock = server.mock("POST", "/capture/").with_status(200).create();

        let client = BlockingPostHog514Client::new("test_key", "test_machine").unwrap();
        let client: Arc<BlockingPostHog514Client> = Arc::new(client);

        let handles: Vec<_> = (0..3)
            .map(|i| {
                let client = Arc::clone(&client);
                thread::spawn(move || client.capture_event(format!("test.event.{}", i), None))
            })
            .collect();

        for handle in handles {
            let result = handle.join().unwrap();
            assert!(result.is_ok());
        }

        mock.assert();
    }

    #[test]
    fn test_blocking_client_timeout() {
        let mut server = Server::new();
        let mock = server.mock("POST", "/capture/").with_status(200).create();

        let client = BlockingPostHog514Client::new("test_key", "test_machine").unwrap();
        let result = client.capture_event("test.timeout", None);

        assert!(matches!(
            result,
            Err(PostHogError::SendEvent {
                source: Some(SendEventErrorKind::Network(_)),
                ..
            })
        ));

        mock.assert();
    }

    #[test]
    fn test_blocking_client_error_handling() {
        let mut server = Server::new();

        // Test rate limiting
        let mock_rate_limit = server.mock("POST", "/capture/").with_status(429).create();

        let client = BlockingPostHog514Client::new("test_key", "test_machine").unwrap();
        let result = client.capture_event("test.rate_limit", None);

        assert!(matches!(
            result,
            Err(PostHogError::SendEvent {
                source: Some(SendEventErrorKind::RateLimited),
                ..
            })
        ));

        mock_rate_limit.assert();

        // Test authentication error
        let mock_auth = server.mock("POST", "/capture/").with_status(401).create();

        let result = client.capture_event("test.auth", None);

        assert!(matches!(
            result,
            Err(PostHogError::SendEvent {
                source: Some(SendEventErrorKind::Authentication),
                ..
            })
        ));

        mock_auth.assert();
    }

    #[test]
    fn test_blocking_client_custom_properties() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/capture/")
            .match_body(mockito::Matcher::Json(json!({
                "api_key": "test_key",
                "event": "test.properties",
                "distinct_id": "test_machine",
                "properties": {
                    "user_id": "123",
                    "plan": "pro"
                }
            })))
            .with_status(200)
            .create();

        let client = BlockingPostHog514Client::new("test_key", "test_machine").unwrap();
        let mut properties = HashMap::new();
        properties.insert("user_id".to_string(), json!("123"));
        properties.insert("plan".to_string(), json!("pro"));

        let result = client.capture_event("test.properties", Some(properties));
        assert!(result.is_ok());

        mock.assert();
    }

    #[test]
    fn test_blocking_client_cli_command() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/capture/")
            .match_body(mockito::Matcher::Json(json!({
                "api_key": "test_key",
                "event": "moose_cli_command",
                "distinct_id": "test_machine",
                "properties": {
                    "command": "test command",
                    "app_version": "1.0.0",
                    "is_developer": true,
                    "environment": "production"
                }
            })))
            .with_status(200)
            .create();

        let client = BlockingPostHog514Client::new("test_key", "test_machine").unwrap();
        let result = client.capture_cli_command("test command", None, None, "1.0.0", true);

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn test_blocking_client_cli_error() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/capture/")
            .match_body(mockito::Matcher::PartialJson(json!({
                "api_key": "test_key",
                "event": "moose_cli_error",
                "distinct_id": "test_machine",
                "properties": {
                    "error": "test error",
                    "error_type": "std::io::Error"
                }
            })))
            .with_status(200)
            .create();

        let client = BlockingPostHog514Client::new("test_key", "test_machine").unwrap();
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let result = client.capture_cli_error(error, None, None);

        assert!(result.is_ok());
        mock.assert();
    }
}
