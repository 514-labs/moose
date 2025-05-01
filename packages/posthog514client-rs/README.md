# posthog514client-rs

A specialized Rust client for PostHog analytics, designed for both CLI applications and general event tracking.

## Features

- Custom event tracking with arbitrary event names and properties
- Both async and blocking (synchronous) APIs
- Built-in CLI command and error tracking
- Project-based analytics support
- Robust error handling with detailed error types
- Build-time API key integration
- Standard 514 properties (app version, developer status, environment)

## Import Guide

### Async Client (Default)

1. Add to Cargo.toml:
```toml
[dependencies]
posthog514client-rs = "0.5.3"
tokio = { version = "1.0", features = ["full"] }  # Required for async usage
```

2. Import the async client and types:
```rust
// Main client
use posthog514client_rs::PostHog514Client;

// Error types
use posthog514client_rs::{
    PostHogError,
    SendEventErrorKind,
    ConfigErrorKind,
    SerializationErrorKind
};

// Event types (for custom events)
use posthog514client_rs::Event514;

// Configuration
use posthog514client_rs::Config;
```

### Blocking Client

1. Add to Cargo.toml with blocking feature:
```toml
[dependencies]
posthog514client-rs = { version = "0.5.3", features = ["blocking"] }
```

2. Import the blocking client and types:
```rust
// Main blocking client
use posthog514client_rs::BlockingPostHog514Client;

// Error types (same as async)
use posthog514client_rs::{
    PostHogError,
    SendEventErrorKind,
    ConfigErrorKind,
    SerializationErrorKind
};

// Event types (same as async)
use posthog514client_rs::Event514;

// Configuration (same as async)
use posthog514client_rs::Config;
```

## Client Configuration

### Required Parameters

```rust
// Basic configuration (required parameters only)
let client = PostHog514Client::new(
    "your-api-key",     // PostHog API key (required)
    "unique-machine-id" // Distinct ID for event tracking (required)
)?;
```

### Optional Configuration

```rust
use std::time::Duration;

// Advanced configuration with all options
let config = Config::new("your-api-key", "https://app.posthog.com")?
    .with_timeout(Duration::from_secs(30))        // Custom request timeout
    .with_retry_attempts(3)                       // Number of retry attempts
    .with_batch_size(20);                        // Event batch size

let client = PostHog514Client::with_config(config)?;
```

### API Key Management

1. **Build-time API Key** (Recommended for CLIs):
```rust
// build.rs
fn main() {
    println!("cargo:rustc-env=POSTHOG_API_KEY={}", env!("POSTHOG_API_KEY"));
}

// main.rs
const API_KEY: &str = env!("POSTHOG_API_KEY");
```

2. **Runtime Environment Variable**:
```rust
// Load from environment at runtime
let client = PostHog514Client::from_env("machine-id")
    .expect("PostHog API key not configured");
```

3. **Configuration File** (for applications):
```rust
use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    posthog_api_key: String,
}

let config: Config = toml::from_str(&fs::read_to_string("config.toml")?)?;
let client = PostHog514Client::new(config.posthog_api_key, "machine-id")?;
```

## Event Types and Properties

### Standard Events

1. **Basic Event**:
```rust
// Simple event with no properties
client.capture_event("user.signup", None).await?;

// Event with properties
let mut props = HashMap::new();
props.insert("plan".to_string(), json!("pro"));
props.insert("referral".to_string(), json!("direct"));
client.capture_event("user.signup", Some(props)).await?;
```

2. **CLI Command Event**:
```rust
client.capture_cli_command(
    "deploy",                    // Command name (required)
    Some("my-project".into()),  // Project name (optional)
    Some(context),              // Additional context (optional)
    "1.0.0".into(),            // App version (required)
    false,                      // Is developer (required)
).await?;
```

3. **Error Event**:
```rust
client.capture_cli_error(
    error,                      // Error impl std::error::Error (required)
    Some("my-project".into()),  // Project name (optional)
    Some(context),              // Additional context (optional)
).await?;
```

### Event Property Structure

All events include these base properties:
```rust
{
    "distinct_id": "machine-id",      // From client initialization
    "timestamp": "2024-03-20T...",    // Automatically added
    "event": "event.name",            // Your event name
    "properties": {
        // Standard properties (when applicable)
        "app_version": "1.0.0",       // From capture_cli_command
        "is_developer": false,        // From capture_cli_command
        "environment": "production",  // Automatically set
        
        // Your custom properties
        "category": "auth",
        "method": "oauth"
    }
}
```

### Property Best Practices

1. **Naming Conventions**:
   - Use lowercase with dots for hierarchy
   - Example: `user.settings.theme`
   - Be consistent across events

2. **Value Types**:
   - Use primitive types where possible
   - For complex types, ensure they're JSON-serializable
   - Avoid large objects or arrays

3. **Common Properties**:
   - Define standard properties for your application
   - Document expected values
   - Validate before sending

## Error Handling

### Error Types

```rust
// Main error type
pub enum PostHogError {
    SendEvent { message: String, source: Option<SendEventErrorKind> },
    Configuration { message: String, source: Option<ConfigErrorKind> },
    Serialization { message: String, source: Option<SerializationErrorKind> }
}

// Network and API errors
pub enum SendEventErrorKind {
    Network(String),    // Connection issues, timeouts
    RateLimited,        // API rate limit exceeded
    Authentication,     // Invalid API key
}

// Configuration errors
pub enum ConfigErrorKind {
    InvalidApiKey,      // Empty or malformed API key
    InvalidUrl(String), // Invalid PostHog instance URL
}

// Serialization errors
pub enum SerializationErrorKind {
    Json,                    // JSON serialization failures
    InvalidPropertyValue,    // Invalid event property values
}
```

### Production Error Handling

1. **Graceful Degradation**:
```rust
pub struct Analytics {
    client: Option<PostHog514Client>,
    error_count: std::sync::atomic::AtomicU32,
}

impl Analytics {
    pub async fn track(&self, event: &str, props: Option<HashMap<String, Value>>) {
        const MAX_ERRORS: u32 = 100;
        
        if let Some(client) = &self.client {
            match client.capture_event(event, props).await {
                Ok(()) => {
                    self.error_count.store(0, Ordering::Relaxed);
                }
                Err(PostHogError::SendEvent { source: Some(SendEventErrorKind::Authentication), .. }) => {
                    log::error!("Invalid API key, disabling analytics");
                    self.client = None;
                }
                Err(e) => {
                    let count = self.error_count.fetch_add(1, Ordering::Relaxed);
                    if count > MAX_ERRORS {
                        log::error!("Too many errors, disabling analytics");
                        self.client = None;
                    } else {
                        log::warn!("Analytics error: {}", e);
                    }
                }
            }
        }
    }
}
```

2. **Retry Strategy**:
```rust
async fn send_with_retry(
    client: &PostHog514Client,
    event: &str,
    props: Option<HashMap<String, Value>>,
    max_retries: u32,
) -> Result<(), PostHogError> {
    let mut attempts = 0;
    loop {
        match client.capture_event(event, props.clone()).await {
            Ok(()) => return Ok(()),
            Err(PostHogError::SendEvent { source: Some(SendEventErrorKind::Network(_)), .. }) => {
                if attempts >= max_retries {
                    return Err(PostHogError::send_event(
                        "Max retries exceeded",
                        Some(SendEventErrorKind::Network("Persistent failure".into())),
                    ));
                }
                tokio::time::sleep(Duration::from_secs(2_u64.pow(attempts))).await;
                attempts += 1;
            }
            // Don't retry authentication or serialization errors
            Err(e) => return Err(e),
        }
    }
}
```

3. **Error Context Preservation**:
```rust
fn track_with_context(
    client: &PostHog514Client,
    event: &str,
    context: &Context,
) -> Result<(), PostHogError> {
    let mut properties = HashMap::new();
    properties.insert("error_context".to_string(), json!({
        "session_id": context.session_id,
        "user_id": context.user_id,
        "environment": context.environment,
    }));

    client
        .capture_event(event, Some(properties))
        .await
        .map_err(|e| {
            log::error!("Analytics error for session {}: {}", context.session_id, e);
            e
        })
}
```

### Error Handling Best Practices

1. **Never Block on Analytics**
   - Use timeouts for all operations
   - Have fallback behavior ready
   - Log errors but don't panic

2. **Preserve Context**
   - Include relevant context in error logging
   - Track error rates and patterns
   - Use structured logging

3. **Smart Retries**
   - Only retry transient failures
   - Use exponential backoff
   - Set maximum retry limits

4. **Circuit Breaking**
   - Track error rates
   - Disable analytics after too many failures
   - Allow manual re-enabling

## Runtime Dependencies

### Async Client
- Requires tokio runtime with the "full" feature set
- Uses reqwest for HTTP requests
- Implements futures traits for async/await support

### Blocking Client
- No async runtime required
- Uses reqwest's blocking client
- Suitable for synchronous applications

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

### Custom Events

```rust
use posthog514client_rs::{PostHog514Client, PostHogError};
use std::collections::HashMap;

async fn track_custom_event() -> Result<(), PostHogError> {
    let client = PostHog514Client::new("your-api-key", "machine-id")?;
    
    // Track a custom event with properties
    let mut properties = HashMap::new();
    properties.insert("user_id".to_string(), json!("123"));
    properties.insert("plan".to_string(), json!("pro"));
    
    client.capture_event("user.upgraded", Some(properties)).await?;
    Ok(())
}
```

With the blocking client:

```rust
use posthog514client_rs::BlockingPostHog514Client;
use std::collections::HashMap;

fn track_custom_event() -> Result<(), PostHogError> {
    let client = BlockingPostHog514Client::new("your-api-key", "machine-id")?;
    
    // Track a custom event with properties
    let mut properties = HashMap::new();
    properties.insert("user_id".to_string(), json!("123"));
    properties.insert("plan".to_string(), json!("pro"));
    
    client.capture_event("user.upgraded", Some(properties))?;
    Ok(())
}
```

### CLI Command Tracking

```rust
use posthog514client_rs::{PostHog514Client, PostHogError};

async fn track_cli_command() -> Result<(), PostHogError> {
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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Synchronous API Guarantees

The blocking client (`BlockingPostHog514Client`) provides the following guarantees:

1. **Thread Safety**: The client is `Clone` and can be safely shared between threads
2. **No Async Runtime**: No tokio or async-std runtime required
3. **Blocking Operations**: All operations block the current thread until completion
4. **Timeout Protection**: Default 10-second timeout on all operations
5. **Resource Management**: Automatic connection pooling and cleanup

### Example: Multi-threaded Usage

```rust
use posthog514client_rs::BlockingPostHog514Client;
use std::thread;

fn main() -> Result<(), PostHogError> {
    let client = BlockingPostHog514Client::new("api-key", "machine-id")?;
    
    // Client can be safely cloned and shared
    let handles: Vec<_> = (0..3).map(|i| {
        let client = client.clone();
        thread::spawn(move || {
            client.capture_event(
                format!("worker.task.{}", i),
                None
            )
        })
    }).collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap()?;
    }

    Ok(())
}
```

### Example: Command Line Tool

```rust
use posthog514client_rs::BlockingPostHog514Client;
use std::collections::HashMap;

fn track_cli_usage(cmd: &str, version: &str) -> Result<(), PostHogError> {
    // Create client with environment-based configuration
    let client = BlockingPostHog514Client::from_env("machine-id")
        .ok_or_else(|| PostHogError::configuration(
            "No API key available",
            Some(ConfigErrorKind::InvalidApiKey)
        ))?;

    // Add command context
    let mut context = HashMap::new();
    context.insert("command".to_string(), json!(cmd));
    context.insert("os".to_string(), json!(std::env::consts::OS));
    context.insert("arch".to_string(), json!(std::env::consts::ARCH));

    // Track command execution
    client.capture_cli_command(
        cmd,
        None,
        Some(context),
        version,
        false
    )
}
```

### Example: Error Tracking Service

```rust
use posthog514client_rs::BlockingPostHog514Client;
use std::sync::Arc;

struct ErrorTracker {
    client: Arc<BlockingPostHog514Client>,
    service_name: String,
}

impl ErrorTracker {
    pub fn new(api_key: &str, service: &str) -> Result<Self, PostHogError> {
        Ok(Self {
            client: Arc::new(BlockingPostHog514Client::new(
                api_key,
                format!("service-{}", service)
            )?),
            service_name: service.to_string(),
        })
    }

    pub fn track_error(&self, error: impl std::error::Error) -> Result<(), PostHogError> {
        let mut context = HashMap::new();
        context.insert("service".to_string(), json!(self.service_name));
        
        self.client.capture_cli_error(
            error,
            None,
            Some(context)
        )
    }
} 