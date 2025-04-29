# posthog514client-rs

A specialized Rust client for PostHog analytics, designed specifically for CLI applications and error tracking.

## Features

- Capture CLI command usage with structured data
- Track CLI errors with detailed context
- Built-in support for project-based analytics
- Async/await support using tokio
- Robust error handling and type safety
- Build-time API key integration for CLI applications

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
posthog514client-rs = "0.1.0"
```

## API Key Management

For CLI applications, it's recommended to bake in the PostHog API key at build time using environment variables. This approach:
- Keeps the API key secure by not storing it in the source code
- Allows different keys for development and production builds
- Prevents end users from needing to provide an API key

### Example Implementation

1. In your `build.rs`:
```rust
fn main() {
    // Make the POSTHOG_API_KEY available at compile time
    println!("cargo:rerun-if-env-changed=POSTHOG_API_KEY");
}
```

2. In your CLI application:
```rust
use posthog514client_rs::{PostHog514Client, PostHogError};

// Define the API key as a build-time constant
const POSTHOG_API_KEY: Option<&str> = option_env!("POSTHOG_API_KEY");

pub struct CliAnalytics {
    client: Option<PostHog514Client>,
    user_id: String,
}

impl CliAnalytics {
    pub fn new(user_id: String) -> Self {
        let client = POSTHOG_API_KEY
            .and_then(|key| PostHog514Client::new(key).ok());

        Self { client, user_id }
    }

    pub async fn track_command(&self, command: &str, project: Option<String>) -> Result<(), PostHogError> {
        // Skip if analytics is disabled (no API key)
        let Some(client) = &self.client else {
            return Ok(());
        };

        client.capture_cli_command(
            &self.user_id,
            command,
            project,
            None,
        ).await
    }
}
```

3. Build your CLI with the API key:
```bash
POSTHOG_API_KEY=your_key cargo build --release
```

## Usage

### Basic Example

```rust
use posthog514client_rs::{PostHog514Client, PostHogError};

async fn example() -> Result<(), PostHogError> {
    // Initialize the client
    let client = PostHog514Client::new("your-api-key")?;
    
    // Track CLI command usage
    client.capture_cli_command(
        "user-id",
        "moose init",
        Some("project-name".to_string()),
        None,
    ).await?;

    Ok(())
}
```

### Error Tracking

```rust
use std::io;
use posthog514client_rs::PostHog514Client;

async fn example_error_tracking() -> Result<(), PostHogError> {
    let client = PostHog514Client::new("your-api-key")?;
    
    // Simulate an error
    let error = io::Error::new(io::ErrorKind::NotFound, "Config file not found");
    
    // Track the error with context
    client.capture_cli_error(
        "user-id",
        error,
        Some("project-name".to_string()),
        None,
    ).await?;

    Ok(())
}
```

## API Reference

### PostHog514Client

- `new(api_key: impl Into<String>) -> Result<Self, PostHogError>`
  - Creates a new PostHog client instance
  - Returns an error if the API key is invalid or client creation fails

- `capture_cli_command(...)` 
  - Captures CLI command execution events
  - Parameters:
    - `distinct_id`: User identifier
    - `command`: The CLI command being executed
    - `project`: Optional project context
    - `context`: Optional additional context as key-value pairs

- `capture_cli_error(...)`
  - Captures CLI error events with full context
  - Parameters:
    - `distinct_id`: User identifier
    - `error`: Any error implementing `std::error::Error`
    - `project`: Optional project context
    - `context`: Optional additional context as key-value pairs

## Error Handling

The library uses a custom `PostHogError` type that provides detailed error information for:
- Configuration errors
- Network/HTTP errors
- Authentication failures
- Rate limiting
- Serialization issues

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 