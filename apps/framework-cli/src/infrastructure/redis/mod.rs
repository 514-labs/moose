/// # Redis Infrastructure
///
/// This module provides a comprehensive Redis client implementation for distributed application concerns
/// such as messaging, leadership election, service presence, and data storage.
///
/// ## Architecture
///
/// The Redis infrastructure is organized into specialized components:
///
/// - **RedisClient**: The main entry point that coordinates all Redis operations
/// - **ConnectionManager**: Handles connection establishment, monitoring, and recovery
/// - **LeadershipManager**: Implements distributed locks for leader election
/// - **PresenceManager**: Maintains service instance health and discoverability
/// - **MessagingManager**: Facilitates pub/sub and queue-based messaging
/// - **MockRedisClient**: Provides local in-memory fallback when Redis is unavailable
///
/// ## Key Features
///
/// - **High Availability**: Graceful degradation when Redis is unavailable
/// - **Thread Safety**: All components are designed for concurrent access
/// - **Automatic Recovery**: Connection monitoring and reconnection
/// - **Performance Optimized**: Using RwLock instead of Mutex where appropriate
///
/// ## Usage Example
///
/// ```rust
/// use framework_cli::infrastructure::redis::{RedisClient, RedisConfig};
///
/// async fn example() -> anyhow::Result<()> {
///     // Create a client with default configuration
///     let client = RedisClient::new("my-service".to_string(), RedisConfig::default()).await?;
///
///     // Start background tasks for presence and connection monitoring
///     client.start_periodic_tasks();
///
///     // Use the client for various Redis operations
///     client.set_with_service_prefix("my-key", "my-value").await?;
///     client.broadcast_message("Hello from my-service").await?;
///
///     // Implement leadership-based functionality
///     if client.attempt_lock("my-lock").await? {
///         // This instance is now the leader for this lock
///         // Perform leader-only operations
///     }
///
///     Ok(())
/// }
/// ```
///
/// ## Error Handling Philosophy
///
/// The Redis module follows a consistent error handling approach:
///
/// - **Critical operations** (locks, data storage) propagate errors to allow proper handling
/// - **Non-critical operations** (messaging, presence) fail gracefully with logged warnings
/// - **Fallback mode** activates automatically when Redis is unavailable
pub mod connection;
pub mod leadership;
pub mod messaging;
pub mod mock;
pub mod presence;
pub mod redis_client;

/// Re-exports the primary types for easier imports.
///
/// This follows the Rust API design best practice of re-exporting
/// the most commonly used types at the module level. Users can import
/// these directly from the redis module:
///
/// ```
/// // Instead of this:
/// use crate::infrastructure::redis::redis_client::RedisClient;
/// use crate::infrastructure::redis::redis_client::RedisConfig;
///
/// // Users can simply write:
/// use crate::infrastructure::redis::{RedisClient, RedisConfig};
/// ```
///
/// This reduces import verbosity and provides a cleaner API surface.
pub use redis_client::RedisClient;
pub use redis_client::RedisConfig;
