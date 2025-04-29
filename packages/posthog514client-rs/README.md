# posthog514client-rs

A Rust client library for PostHog analytics.

## Features

- Async/await support
- Type-safe event capture
- Batch event support
- Error handling and retries
- Tracing support

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
posthog514client-rs = "0.1.0"
```

Basic example:

```rust
use posthog514client_rs::{PostHogClient, Event};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PostHogClient::new("your-api-key", "https://app.posthog.com");
    
    client.capture(Event::new("event_name")
        .set_distinct_id("user_123")
        .add_property("key", "value"))
        .await?;
    
    Ok(())
}
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 