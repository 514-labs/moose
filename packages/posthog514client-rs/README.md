# posthog514client-rs

A specialized Rust client for PostHog analytics, designed specifically for CLI applications and error tracking.

## Features

- Capture CLI command usage with structured data
- Track CLI errors with detailed context
- Built-in support for project-based analytics
- Async/await support using tokio
- Robust error handling and type safety
- Build-time API key integration for CLI applications
- Standard 514 properties (app version, developer status, environment)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
posthog514client-rs = "0.3.0"
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

pub struct CliAnalytics {
    client: Option<PostHog514Client>,
    machine_id: String,
}

impl CliAnalytics {
    pub fn new(machine_id: String) -> Self {
        // Will first check for build-time API key, then runtime environment variable
        let client = PostHog514Client::from_env(machine_id.clone());

        Self { client, machine_id }
    }

    pub async fn track_command(
        &self,
        command: &str,
        project: Option<String>,
        app_version: String,
        is_developer: bool,
    ) -> Result<(), PostHogError> {
        // Skip if analytics is disabled (no API key)
        let Some(client) = &self.client else {
            return Ok(());
        };

        client.capture_cli_command(
            command,
            project,
            None,
            app_version,
            is_developer,
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
    // Initialize the client with machine ID
    let client = PostHog514Client::new("your-api-key", "machine-id")?;
    
    // Track CLI command usage
    client.capture_cli_command(
        "moose init",
        Some("project-name".to_string()),
        None,
        "1.0.0",
        false,
    ).await?;

    Ok(())
}
```

### Error Tracking

```rust
use std::io;
use posthog514client_rs::PostHog514Client;

async fn example_error_tracking() -> Result<(), PostHogError> {
    let client = PostHog514Client::new("your-api-key", "machine-id")?;
    
    // Simulate an error
    let error = io::Error::new(io::ErrorKind::NotFound, "Config file not found");
    
    // Track the error with context
    client.capture_cli_error(
        error,
        Some("project-name".to_string()),
        None,
    ).await?;

    Ok(())
}
```

## API Reference

### PostHog514Client

- `new(api_key: impl Into<String>, machine_id: impl Into<String>) -> Result<Self, PostHogError>`
  - Creates a new PostHog client instance
  - Returns an error if the API key is invalid or client creation fails

- `from_env(machine_id: impl Into<String>) -> Option<Self>`
  - Creates a client using the API key from environment
  - First checks for build-time API key, then runtime environment variable
  - Returns None if no API key is available

- `capture_cli_command(...)` 
  - Captures CLI command execution events
  - Parameters:
    - `command`: The CLI command being executed
    - `project`: Optional project context
    - `context`: Optional additional context as key-value pairs
    - `app_version`: Version of the application
    - `is_developer`: Whether the user is a developer

- `capture_cli_error(...)`
  - Captures CLI error events with full context
  - Parameters:
    - `error`: Any error implementing `std::error::Error`
    - `project`: Optional project context
    - `context`: Optional additional context as key-value pairs

## Properties

The client sends the following properties with each event:

### Core Properties
- `app_version`: Version of the CLI application
- `is_developer`: Whether the user is a developer
- `environment`: The environment (e.g., "production")
- `project`: The project context if available

### Custom Properties
Additional properties can be added via the context parameter.

## Error Handling

The library uses a custom `PostHogError` type that provides detailed error information for:
- Configuration errors
- Network/HTTP errors
- Authentication failures
- Rate limiting
- Serialization issues

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 