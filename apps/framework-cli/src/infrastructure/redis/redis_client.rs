/// Moose Application-level Redis Client
///
/// This module offers a wrapper around the Redis crate to provide support for
/// multi-moose distributed concerns such as interprocess communication and data
/// processing coordination via leadership management and presence tracking.
///
/// CONCURRENCY NOTES:
///
/// The async `ConnectionManager` from the Redis crate implements `Send` and
/// is therefore safe to move between threads. However, its API requires
/// mutable access (`&mut self`) for sending commands, which means that
/// concurrent command execution is not supported out of the box.
///
/// To safely use the connection in an asynchronous context, we wrap it in a
/// Tokio `Mutex`. This approach allows asynchronous tasks to wait for access
/// without blocking the executor, ensuring that only one task can perform
/// operations on the connection at a time. In this
/// scenario, an `RwLock` would not offer any advantage because all operations
/// require mutable access, leaving no opportunity for concurrent read-only
/// access.
///
/// Using a Tokio `Mutex` maintains thread safety and provides the necessary
/// mutable access control for our Redis component.
///
/// Additional notes:
///
/// - We replaced `Arc<TokioMutex<Option<JoinHandle<()>>>>` with
///   `Arc<RwLock<Option<JoinHandle<()>>>>` for all task handles:
///
/// ```
/// Before
/// pub listener_task: Arc<TokioMutex<Option<JoinHandle<()>>>>,
/// pub presence_task: Arc<TokioMutex<Option<JoinHandle<()>>>>,
/// pub connection_monitor_task: Arc<TokioMutex<Option<JoinHandle<()>>>>,
///
/// After
/// pub listener_task: Arc<RwLock<Option<JoinHandle<()>>>>,
/// pub presence_task: Arc<RwLock<Option<JoinHandle<()>>>>,
/// pub connection_monitor_task: Arc<RwLock<Option<JoinHandle<()>>>>,
/// ```
///
/// This change allows for concurrent reads of the task handles, which is
/// beneficial since these are primarily read-only after initialization.
/// We only need to acquire a write lock when we're setting or taking the
/// task handle.
///
/// - We replaced `Arc<tokio::sync::Mutex<std::collections::HashMap<String, (String, i64)>>>`
///   with `Arc<RwLock<std::collections::HashMap<String, (String, i64)>>>`
///   for the locks map:
///
/// ```
/// Before
/// pub locks: Arc<tokio::sync::Mutex<std::collections::HashMap<String, (String, i64)>>>,
///
/// After
/// pub locks: Arc<RwLock<std::collections::HashMap<String, (String, i64)>>>,
/// ```
///
/// This change allows for concurrent reads of the locks map, which is
/// beneficial since most operations on the locks map are read operations
/// (checking if a lock exists, getting lock information). We only need
/// to acquire a write lock when we're adding or removing locks.
///
use futures::FutureExt;
use futures::StreamExt;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex as TokioMutex;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time;

// Type alias for complex callback types
type MessageCallback = Arc<dyn Fn(String) + Send + Sync>;

/// # Redis Configuration
///
/// This section defines the configuration structure for Redis connections.
/// The `RedisConfig` struct provides the necessary parameters to establish
/// and maintain connections to Redis servers.
///
/// ## Configuration Options
///
/// - `url`: The Redis connection URL in the format:
///          `redis://[username:password@]host[:port][/db]`.
///   Defaults to `redis://127.0.0.1:6379` (local Redis server on default port).
///
/// - `key_prefix`: A namespace prefix added to all Redis keys to prevent
///   collisions when multiple services share the same Redis instance.
///   Defaults to `"MS"`.
///
/// ## Usage
///
/// ```plaintext
/// let config = RedisConfig {
///     url: "redis://redis.example.com:6379".to_string(),
///     key_prefix: "my-service".to_string(),
/// };
/// ```
///
/// Or use defaults
///
/// ```plaintext
/// let default_config = RedisConfig::default();
/// ```
///
/// Create a Redis client with the configuration
///
/// ```plaintext
/// let redis_client = RedisClient::new("service-name", config).await?;
/// ```
///
/// ## Environment Integration
///
/// The Redis configuration can be loaded from environment variables or
/// configuration files using the `config` crate's deserialization
/// capabilities, as the struct implements `Serialize` and `Deserialize`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisConfig {
    #[serde(default = "RedisConfig::default_url")]
    pub url: String,
    #[serde(default = "RedisConfig::default_key_prefix")]
    pub key_prefix: String,
}

impl RedisConfig {
    /// Returns the default Redis URL.
    ///
    /// This method provides the default connection URL for Redis when none is
    /// specified. It points to a local Redis instance running on the standard
    /// port.
    ///
    /// # Returns
    ///
    /// A string containing the default Redis URL: `redis://127.0.0.1:6379`
    pub fn default_url() -> String {
        "redis://127.0.0.1:6379".to_string()
    }

    /// Returns the default key prefix for Redis keys.
    ///
    /// This method provides the default namespace prefix that will be
    /// prepended to all Redis keys to prevent collisions when multiple
    /// services share the same Redis instance.
    ///
    /// # Returns
    ///
    /// A string containing the default key prefix: `"MS"`
    pub fn default_key_prefix() -> String {
        "MS".to_string()
    }
}

/// Implements the Default trait for RedisConfig.
///
/// This allows creating a RedisConfig with default values using
/// `RedisConfig::default()`.  The default configuration uses a local
/// Redis instance and the "MS" key prefix.
impl Default for RedisConfig {
    fn default() -> Self {
        RedisConfig {
            url: RedisConfig::default_url(),
            key_prefix: RedisConfig::default_key_prefix(),
        }
    }
}

/// # Connection Module
///
/// This module handles Redis connection management, including connection
/// establishment, health monitoring, and automatic reconnection. It provides
/// a thread-safe wrapper around the Redis ConnectionManager to ensure
/// reliable communication with Redis servers.
///
/// ## Key Components
///
/// - `ConnectionState`: Enum representing the possible states of a Redis connection.
///
/// - `ConnectionManagerWrapper`: A wrapper around Redis ConnectionManager that provides:
///   - Thread-safe access to Redis connections via Tokio mutexes
///   - Connection health monitoring via periodic pings
///   - Automatic reconnection with exponential backoff
///   - Separate connections for regular commands and pub/sub operations
///
/// ## Connection Lifecycle
///
/// 1. **Initialization**: Connections are established when creating a new
///                        `ConnectionManagerWrapper`
/// 2. **Health Monitoring**: Connections are periodically checked with ping commands
/// 3. **Failure Detection**: Failed pings trigger the reconnection process
/// 4. **Reconnection**: Automatic reconnection attempts with exponential backoff
///
/// ## Usage Example
///
/// ```plaintext
/// let client = redis::Client::open("redis://127.0.0.1:6379")?;
/// let connection_wrapper = ConnectionManagerWrapper::new(&client).await?;
///
/// Check connection health
/// if !connection_wrapper.ping().await {
///     Connection is unhealthy, but reconnection will be attempted automatically
/// }
/// ```
///
/// ## Thread Safety
///
/// The ConnectionManagerWrapper is designed to be thread-safe and can be
/// safely shared across multiple asynchronous tasks. It uses Tokio mutexes
/// to ensure that only one task can access the connection at a time,
/// preventing race conditions.
pub mod connection {
    use super::*;

    /// Represents the possible states of a Redis connection.
    ///
    /// This enum is used to track and communicate the current state of the
    /// connection to Redis, which is useful for monitoring and diagnostics.
    pub enum ConnectionState {
        /// The connection to Redis is established and operational.
        Connected,
        /// The connection to Redis has been lost or closed.
        Disconnected,
        /// The system is currently attempting to reestablish a lost connection.
        Reconnecting,
    }

    /// A thread-safe wrapper around Redis ConnectionManager.
    ///
    /// This struct provides a wrapper around the Redis ConnectionManager that
    /// ensures thread safety and adds features like connection health
    /// monitoring and automatic reconnection with exponential backoff.
    #[derive(Clone)]
    pub struct ConnectionManagerWrapper {
        /// The main connection used for regular Redis commands.
        pub connection: Arc<TokioMutex<ConnectionManager>>,
        /// A separate connection dedicated to pub/sub operations.
        pub pub_sub: Arc<TokioMutex<ConnectionManager>>,
        /// Atomic flag indicating the connection state (true = connected).
        pub state: Arc<AtomicBool>, // true = connected
    }

    impl ConnectionManagerWrapper {
        /// Creates a new ConnectionManagerWrapper with the provided
        /// Redis client.
        ///
        /// This method establishes two separate connections to Redis:
        /// 1. A main connection for regular Redis commands
        /// 2. A dedicated connection for pub/sub operations
        ///
        /// Both connections are wrapped in Tokio mutexes to ensure thread
        /// safety. The connection state is initially set to connected (true).
        ///
        /// # Parameters
        ///
        /// - `client` - A reference to a Redis Client instance used to
        ///    establish connections
        ///
        /// # Returns
        ///
        /// - `Result<Self, RedisError>` - A new ConnectionManagerWrapper or a
        ///   Redis error
        ///
        /// # Errors
        ///
        /// This method will return an error if it fails to establish either
        /// connection to Redis.
        pub async fn new(client: &Client) -> Result<Self, RedisError> {
            // Create connection manager for normal operations
            let conn = client.get_connection_manager().await?;
            let pub_sub_conn = client.get_connection_manager().await?;
            Ok(ConnectionManagerWrapper {
                connection: Arc::new(TokioMutex::new(conn)),
                pub_sub: Arc::new(TokioMutex::new(pub_sub_conn)),
                state: Arc::new(AtomicBool::new(true)),
            })
        }

        /// Checks the health of the Redis connection by sending a PING command.
        ///
        /// This method attempts to send a PING command to Redis with a timeout
        /// of 2 seconds. If the PING succeeds, it returns true. If it fails or
        /// times out, it updates the connection state to disconnected (false)
        /// and returns false.
        ///
        /// # Returns
        ///
        /// - `bool` - true if the connection is healthy, false otherwise
        pub async fn ping(&self) -> bool {
            let timeout_future = time::timeout(
                Duration::from_secs(2),
                AssertUnwindSafe(async {
                    redis::cmd("PING")
                        .query_async::<_, String>(&mut *self.connection.lock().await)
                        .await
                })
                .catch_unwind(),
            );

            match timeout_future.await {
                Ok(Ok(Ok(_response))) => true,
                _ => {
                    self.state.store(false, Ordering::SeqCst);
                    false
                }
            }
        }

        /// Attempts to reconnect to Redis with exponential backoff.
        ///
        /// This method is called when a connection failure is detected. It
        /// attempts to reestablish the connection with an exponential backoff
        /// strategy:
        ///
        /// 1. Initial backoff of 5 seconds
        /// 2. Doubles the backoff time after each failed attempt
        /// 3. Caps the maximum backoff at 60 seconds
        ///
        /// Once a connection is successfully established, it updates the
        /// connection state to connected (true) and returns.
        ///
        /// # Parameters
        ///
        /// - `client` - A reference to a Redis Client instance used to
        ///   establish a new connection
        /// - `_config` - A reference to the Redis configuration
        ///   (currently unused)
        pub async fn attempt_reconnection(&self, client: &Client, _config: &RedisConfig) {
            let mut backoff = 5;
            while !self.state.load(Ordering::SeqCst) {
                time::sleep(Duration::from_secs(backoff)).await;
                if let Ok(new_conn) = client.get_connection_manager().await {
                    let mut conn_guard = self.connection.lock().await;
                    *conn_guard = new_conn;
                    self.state.store(true, Ordering::SeqCst);
                    break;
                } else {
                    backoff = std::cmp::min(backoff * 2, 60);
                }
            }
        }
    }
}

/// # Leadership Module
///
/// This module implements distributed leadership election and lock management
/// using Redis. It provides mechanisms for services to coordinate and elect
/// a leader among multiple instances, ensuring that critical operations are
/// performed by only one instance at a time.
///
/// ## Key Features
///
/// - **Distributed Locking**: Atomic lock acquisition and release using Redis
///   Lua scripts
/// - **Leader Election**: Allows multiple service instances to elect a leader
/// - **Lock Renewal**: Supports extending lock ownership with TTL (Time To Live)
/// - **Lock Verification**: Methods to check if the current instance owns a
///   specific lock
///
/// ## Lock Lifecycle
///
/// 1. **Acquisition**: A service instance attempts to acquire a lock with a TTL
/// 2. **Verification**: The instance can verify it still owns the lock
/// 3. **Renewal**: The lock can be renewed to extend its lifetime
/// 4. **Release**: The lock is explicitly released or expires after its TTL
///
/// ## Implementation Details
///
/// The module uses Redis Lua scripts to ensure atomicity of lock operations.
/// Each lock has:
/// - A unique key in Redis
/// - An instance ID identifying the lock owner
/// - A TTL (Time To Live) to prevent deadlocks if an instance fails
///
/// ## Usage Example
///
/// ```plaintext
/// Create a leadership manager
/// let leadership = LeadershipManager::new("instance-123".to_string(),
/// redis_config);
///
/// Attempt to acquire a lock
/// let lock_acquired = leadership.attempt_lock(connection, "critical-operation", 30).await;
///
/// if lock_acquired {
///     Perform leader-only operations
///     
///     Renew the lock to maintain leadership
///     leadership.renew_lock(connection, "critical-operation", "instance-123", 30).await?;
///     
///     Release the lock when done
///     leadership.release_lock(connection, "critical-operation", "instance-123").await?;
/// }
/// ```
///
/// ## Thread Safety
///
/// The LeadershipManager is designed to be thread-safe and can be safely
/// shared across multiple asynchronous tasks. The Redis operations are atomic,
/// ensuring consistent lock state across distributed instances.
pub mod leadership {
    use super::*;
    use redis::Script;

    /// Manager for distributed leadership and lock coordination.
    ///
    /// This struct provides methods to acquire, verify, renew, and release
    /// distributed locks across multiple service instances using Redis.
    pub struct LeadershipManager {
        /// Unique identifier for this service instance.
        pub instance_id: String,
        /// Redis configuration for connecting to the Redis server.
        pub redis_config: RedisConfig,
    }

    impl LeadershipManager {
        /// Creates a new LeadershipManager instance.
        ///
        /// # Parameters
        ///
        /// - `instance_id` - A unique identifier for this service instance
        /// - `config` - Redis configuration for connecting to the Redis server
        ///
        /// # Returns
        ///
        /// A new LeadershipManager instance configured with the provided
        /// parameters
        pub fn new(instance_id: String, config: RedisConfig) -> Self {
            LeadershipManager {
                instance_id,
                redis_config: config,
            }
        }

        /// Attempts to acquire a distributed lock.
        ///
        /// This method tries to acquire a lock with the following logic:
        /// - If the lock doesn't exist, it's created with this instance as the owner
        /// - If the lock exists and is owned by this instance, the TTL is refreshed
        /// - If the lock exists and is owned by another instance, acquisition fails
        ///
        /// The operation is performed atomically using a Lua script.
        ///
        /// # Parameters
        ///
        /// - `conn` - Redis connection manager
        /// - `lock_key` - Unique identifier for the lock
        /// - `ttl` - Time To Live in seconds for the lock
        ///
        /// # Returns
        ///
        /// - `bool` - true if the lock was acquired or already owned, false
        ///   otherwise
        pub async fn attempt_lock(
            &self,
            conn: Arc<TokioMutex<ConnectionManager>>,
            lock_key: &str,
            ttl: i64,
        ) -> bool {
            let script = Script::new(
                r#"
                local current = redis.call('GET', KEYS[1])
                if not current then
                    redis.call('SET', KEYS[1], ARGV[1])
                    redis.call('EXPIRE', KEYS[1], ARGV[2])
                    return 1
                elseif current == ARGV[1] then
                    redis.call('EXPIRE', KEYS[1], ARGV[2])
                    return 1
                else
                    return 0
                end
                "#,
            );
            let instance = self.instance_id.clone();
            let result = {
                let mut conn_guard = conn.lock().await;
                script
                    .key(lock_key)
                    .arg(instance)
                    .arg(ttl)
                    .invoke_async::<_, i32>(&mut *conn_guard)
                    .await
            };
            matches!(result, Ok(val) if val == 1)
        }

        /// Renews an existing lock by extending its TTL.
        ///
        /// This method extends the TTL of a lock only if it's currently owned
        /// by the specified instance. The operation is performed atomically
        /// using a Lua script.
        ///
        /// # Parameters
        ///
        /// - `conn` - Redis connection manager
        /// - `lock_key` - Unique identifier for the lock
        /// - `instance_id` - The instance ID that should own the lock
        /// - `ttl` - New Time To Live in seconds for the lock
        ///
        /// # Returns
        ///
        /// - `anyhow::Result<bool>` - Ok(true) if the lock was renewed,
        ///   Ok(false) if not owned by this instance
        ///
        /// # Errors
        ///
        /// Returns an error if the Redis operation fails
        pub async fn renew_lock(
            &self,
            conn: Arc<TokioMutex<ConnectionManager>>,
            lock_key: &str,
            instance_id: &str,
            ttl: i64,
        ) -> anyhow::Result<bool> {
            let script = redis::Script::new(
                "if redis.call('GET', KEYS[1]) == ARGV[1] then return redis.call('EXPIRE', KEYS[1], ARGV[2]) else return 0 end",
            );
            let result: i32 = script
                .key(lock_key)
                .arg(instance_id)
                .arg(ttl)
                .invoke_async::<_, i32>(&mut *conn.lock().await)
                .await?;
            Ok(result == 1)
        }

        /// Releases a lock if it's owned by the specified instance.
        ///
        /// This method deletes a lock only if it's currently owned by the
        /// specified instance. The operation is performed atomically using
        /// a Lua script.
        ///
        /// # Parameters
        ///
        /// - `conn` - Redis connection manager
        /// - `lock_key` - Unique identifier for the lock
        /// - `instance_id` - The instance ID that should own the lock
        ///
        /// # Returns
        ///
        /// - `anyhow::Result<()>` - Ok(()) if the operation was successful
        ///
        /// # Errors
        ///
        /// Returns an error if the Redis operation fails
        pub async fn release_lock(
            &self,
            conn: Arc<TokioMutex<ConnectionManager>>,
            lock_key: &str,
            instance_id: &str,
        ) -> anyhow::Result<()> {
            let script = redis::Script::new(
                "if redis.call('GET', KEYS[1]) == ARGV[1] then\n return redis.call('DEL', KEYS[1])\nelse\n return 0\nend"
            );
            let _result: () = script
                .key(lock_key)
                .arg(instance_id)
                .invoke_async(&mut *conn.lock().await)
                .await?;
            Ok(())
        }

        /// Checks if a lock is currently owned by the specified instance.
        ///
        /// This method verifies if a lock exists and is owned by the specified
        /// instance without modifying the lock. The operation is performed
        /// atomically using a Lua script.
        ///
        /// # Parameters
        ///
        /// - `conn` - Redis connection manager
        /// - `lock_key` - Unique identifier for the lock
        /// - `instance_id` - The instance ID to check ownership against
        ///
        /// # Returns
        ///
        /// - `anyhow::Result<bool>` - Ok(true) if the lock is owned by the
        ///   instance, Ok(false) otherwise
        ///
        /// # Errors
        ///
        /// Returns an error if the Redis operation fails
        pub async fn has_lock(
            &self,
            conn: Arc<TokioMutex<ConnectionManager>>,
            lock_key: &str,
            instance_id: &str,
        ) -> anyhow::Result<bool> {
            let script = redis::Script::new(
                "if redis.call('GET', KEYS[1]) == ARGV[1] then return 1 else return 0 end",
            );
            let result: i32 = script
                .key(lock_key)
                .arg(instance_id)
                .invoke_async::<_, i32>(&mut *conn.lock().await)
                .await?;
            Ok(result == 1)
        }
    }
}

/// # Presence Module
///
/// This module implements service instance presence tracking using Redis.
/// It allows service instances to register their presence and maintain a
/// heartbeat, enabling other components to detect active instances and handle
/// instance failures.
///
/// ## Key Features
///
/// - **Instance Registration**: Services register their presence in Redis
/// - **Heartbeat Mechanism**: Periodic updates with TTL to maintain presence
/// - **Automatic Expiration**: TTL-based keys automatically expire when
///   instances fail
/// - **Instance Discovery**: Other services can discover active instances
///
/// ## Presence Lifecycle
///
/// 1. **Registration**: A service instance registers its presence with a
///    unique key
/// 2. **Heartbeat**: The instance periodically updates its presence key with
///    a new TTL
/// 3. **Expiration**: If an instance fails, its presence key expires after
///    the TTL
/// 4. **Discovery**: Other instances can query Redis to find active instances
///
/// ## Implementation Details
///
/// The module uses Redis TTL keys to track instance presence:
/// - Each instance has a unique presence key in Redis
/// - The key contains a timestamp of the last heartbeat
/// - The key has a TTL (typically 10 seconds) that must be refreshed
/// - If an instance fails to update its key, it expires automatically
///
/// ## Usage Example
///
/// ```plaintext
/// Create a presence manager
/// let presence = PresenceManager::new("instance-123".to_string(), "my-service".to_string());
///
/// Register and maintain presence
/// tokio::spawn(async move {
///     let mut interval = tokio::time::interval(Duration::from_secs(5));
///     loop {
///         interval.tick().await;
///         presence.update_presence(connection).await?;
///     }
/// });
/// ```
///
/// ## Thread Safety
///
/// The PresenceManager is designed to be thread-safe and can be safely shared
/// across multiple asynchronous tasks. It is also cloneable, allowing it to be
/// used in multiple concurrent contexts.
pub mod presence {
    use super::*;
    use anyhow::Context;

    /// Manager for service instance presence tracking.
    ///
    /// This struct provides methods to register and maintain the presence
    /// of a service instance in a distributed system using Redis TTL keys.
    /// It is cloneable to allow sharing across multiple tasks.
    #[derive(Clone)]
    pub struct PresenceManager {
        /// Unique identifier for this service instance.
        pub instance_id: String,
        /// Prefix for Redis keys to prevent collisions.
        pub key_prefix: String,
    }

    impl PresenceManager {
        /// Creates a new PresenceManager instance.
        ///
        /// # Parameters
        ///
        /// - `instance_id` - A unique identifier for this service instance
        /// - `key_prefix` - Prefix for Redis keys to prevent collisions
        ///
        /// # Returns
        ///
        /// A new PresenceManager instance configured with the provided
        /// parameters
        pub fn new(instance_id: String, key_prefix: String) -> Self {
            PresenceManager {
                instance_id,
                key_prefix,
            }
        }

        /// Generates the Redis key used for tracking this instance's presence.
        ///
        /// The key follows the format: `{key_prefix}::{instance_id}::presence`
        ///
        /// # Returns
        ///
        /// A string containing the formatted presence key
        fn presence_key(&self) -> String {
            format!("{}::{}::presence", self.key_prefix, self.instance_id)
        }

        /// Updates the instance's presence in Redis with a new TTL.
        ///
        /// This method:
        /// 1. Generates the presence key for this instance
        /// 2. Gets the current timestamp
        /// 3. Sets the key in Redis with the timestamp as value
        /// 4. Applies a TTL of 10 seconds to the key
        ///
        /// This should be called periodically (typically every few seconds)
        /// to maintain the instance's presence. If the instance fails to
        /// update its presence, the key will expire after the TTL.
        ///
        /// # Parameters
        ///
        /// - `conn` - Redis connection manager
        ///
        /// # Returns
        ///
        /// - `anyhow::Result<()>` - Ok(()) if the operation was successful
        ///
        /// # Errors
        ///
        /// Returns an error if:
        /// - Failed to get the current time
        /// - Redis operation fails
        pub async fn update_presence(
            &self,
            conn: Arc<TokioMutex<ConnectionManager>>,
        ) -> anyhow::Result<()> {
            let key = self.presence_key();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("Failed to get current time")?
                .as_secs();
            {
                let mut conn_guard = conn.lock().await;
                let _: () = conn_guard.set_ex(&key, now, 10).await?;
            }
            Ok(())
        }
    }
}

/// # Messaging Module
///
/// This module implements pub/sub messaging between service instances using
/// Redis. It provides a simple and efficient way for instances to communicate
/// with each other through broadcast messages or targeted instance-specific
/// messages.
///
/// ## Key Features
///
/// - **Publish/Subscribe**: Implements the Redis pub/sub pattern
/// - **Targeted Messaging**: Send messages to specific service instances
/// - **Broadcast Messaging**: Send messages to all instances
/// - **Channel Management**: Handles channel naming and message routing
///
/// ## Messaging Patterns
///
/// The module supports two primary messaging patterns:
///
/// 1. **Direct Messaging**: Messages sent to a specific instance
///    - Channel format: `{key_prefix}::{instance_id}::msgchannel`
///    - Only the targeted instance receives the message
///
/// 2. **Broadcast Messaging**: Messages sent to all instances
///    - Channel format: `{key_prefix}::msgchannel`
///    - All instances subscribed to the channel receive the message
///
/// ## Implementation Details
///
/// The module uses Redis pub/sub channels to implement messaging:
/// - Each instance subscribes to its own instance-specific channel
/// - All instances subscribe to a common broadcast channel
/// - Messages are published to these channels using the Redis PUBLISH command
/// - Subscribers receive messages asynchronously through callbacks
///
/// ## Usage Example
///
/// ```plaintext
/// Create a messaging manager
/// let messaging = MessagingManager::new("my-service".to_string());
///
/// Send a message to a specific instance
/// messaging.publish_message(
///     connection,
///     "instance-123",
///     "Hello, specific instance!"
/// ).await?;
///
/// Broadcast a message to all instances
/// messaging.publish_message(
///     connection,
///     "msgchannel",
///     "Hello, all instances!"
/// ).await?;
/// ```
///
/// ## Thread Safety
///
/// The MessagingManager is designed to be thread-safe and can be safely shared
/// across multiple asynchronous tasks. The Redis pub/sub operations are handled
/// by the Redis connection manager, which ensures thread safety.
pub mod messaging {
    use super::*;

    /// Manager for pub/sub messaging between service instances.
    ///
    /// This struct provides methods to publish messages to specific instances
    /// or broadcast messages to all instances using Redis pub/sub channels.
    pub struct MessagingManager {
        /// Prefix for Redis keys and channels to prevent collisions.
        pub key_prefix: String,
    }

    impl MessagingManager {
        /// Creates a new MessagingManager instance.
        ///
        /// # Parameters
        ///
        /// - `key_prefix` - Prefix for Redis keys and channels to prevent
        ///   collisions
        ///
        /// # Returns
        ///
        /// A new MessagingManager instance configured with the provided key
        /// prefix
        pub fn new(key_prefix: String) -> Self {
            MessagingManager { key_prefix }
        }

        /// Generates the Redis channel name for a specific target.
        ///
        /// This method formats the channel name using the pattern:
        /// `{key_prefix}::{target}::msgchannel`
        ///
        /// The target can be:
        /// - An instance ID for direct messaging
        /// - "msgchannel" for broadcast messaging
        ///
        /// # Parameters
        ///
        /// - `target` - The target identifier (instance ID or "msgchannel")
        ///
        /// # Returns
        ///
        /// A string containing the formatted channel name
        pub fn channel_for(&self, target: &str) -> String {
            format!("{}::{}::msgchannel", self.key_prefix, target)
        }

        /// Publishes a message to a Redis pub/sub channel.
        ///
        /// This method sends a message to the specified target channel.
        /// The target can be an instance ID for direct messaging or
        /// a broadcast channel identifier for sending to all instances.
        ///
        /// # Parameters
        ///
        /// - `conn` - Redis connection manager
        /// - `target` - The target identifier (instance ID or channel name)
        /// - `message` - The message content to publish
        ///
        /// # Returns
        ///
        /// - `anyhow::Result<()>` - Ok(()) if the operation was successful
        ///
        /// # Errors
        ///
        /// Returns an error if the Redis publish operation fails
        pub async fn publish_message(
            &self,
            conn: Arc<TokioMutex<ConnectionManager>>,
            target: &str,
            message: &str,
        ) -> anyhow::Result<()> {
            let channel = self.channel_for(target);
            let mut conn_guard = conn.lock().await;
            let _: () = conn_guard.publish(&channel, message).await?;
            Ok(())
        }
    }
}

/// # Mock Module
///
/// This module provides a fallback implementation of Redis functionality for
/// testing and situations where a Redis server is unavailable. It implements
/// an in-memory version of the Redis operations used by the client, allowing
/// the application to continue functioning even without a Redis connection.
///
/// ## Key Features
///
/// - **In-Memory Storage**: Uses a thread-safe HashMap to simulate Redis
///   storage
/// - **Fallback Mechanism**: Automatically used when Redis connection fails
/// - **API Compatibility**: Implements the same interface as the real Redis
///   client
/// - **Testing Support**: Enables unit testing without requiring a Redis server
///
/// ## Implementation Details
///
/// The mock implementation:
/// - Uses a standard Mutex-protected HashMap for thread-safe storage
/// - Simulates key-value operations, locks, and queue operations
/// - Provides simplified versions of Redis commands
/// - Does not implement all Redis features (only those needed by the client)
///
/// ## Usage
///
/// The mock client is typically used in two scenarios:
///
/// 1. **Automatic Fallback**: When Redis connection fails, the client
///    automatically switches to the mock implementation:
///
///    ```rust
///    // If Redis connection fails, fallback is used automatically
///    let redis_client = RedisClient::new("service-name", config).await?;
///    // Operations will use mock if real Redis is unavailable
///    redis_client.post_queue_message("message", None).await?;
///    ```
///
/// 2. **Testing**: For unit tests, the mock can be explicitly created:
///
///    ```plaintext
///    Create a mock client for testing
///    let mock_client = MockRedisClient::new();
///    mock_client.post_queue_message("queue", "test message").await?;
///    ```
///
/// ## Limitations
///
/// The mock implementation has several limitations:
/// - No persistence (data is lost when the application restarts)
/// - Limited Redis command support (only implements what's needed)
/// - No distributed capabilities (only works within a single process)
/// - Simplified locking behavior (not fully Redis-compatible)
pub mod mock {
    use anyhow::Result;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    /// An in-memory mock implementation of Redis client functionality.
    ///
    /// This struct provides a fallback implementation when a Redis server is
    /// unavailable or for testing purposes. It uses a thread-safe HashMap to
    /// simulate Redis storage and implements simplified versions of the Redis
    /// operations used by the client.
    #[derive(Default, Clone, Debug)]
    pub struct MockRedisClient {
        /// Thread-safe in-memory storage that simulates Redis.
        /// Using RwLock instead of Mutex to allow concurrent reads, which
        /// improves performance in read-heavy scenarios common in Redis
        /// usage patterns.
        pub store: Arc<RwLock<HashMap<String, String>>>,
    }

    impl MockRedisClient {
        /// Creates a new MockRedisClient instance with an empty storage.
        ///
        /// # Returns
        ///
        /// A new MockRedisClient instance with an initialized empty HashMap
        pub fn new() -> Self {
            Self {
                store: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        /// Simulates updating the presence of a service instance.
        ///
        /// This method stores the presence information in the in-memory
        /// HashMap, mimicking the behavior of the real Redis presence update
        /// operation.
        ///
        /// # Parameters
        ///
        /// - `key` - The presence key for the instance
        /// - `value` - The timestamp value to store
        ///
        /// # Returns
        ///
        /// - `Result<()>` - Ok if the operation was successful
        pub async fn update_presence(&self, key: &str, value: u64) -> Result<()> {
            let mut store = self.store.write().await;
            store.insert(key.to_string(), value.to_string());
            Ok(())
        }

        /// Simulates attempting to acquire a distributed lock.
        ///
        /// This method checks if a lock exists and is owned by the specified
        /// instance. If the lock doesn't exist, it creates it with the
        /// specified instance as owner.
        ///
        /// # Parameters
        ///
        /// - `key` - The lock key
        /// - `instance_id` - The instance ID attempting to acquire the lock
        ///
        /// # Returns
        ///
        /// - `bool` - true if the lock was acquired or already owned, false
        ///   otherwise
        pub async fn attempt_lock(&self, key: &str, instance_id: &str) -> bool {
            let mut store = self.store.write().await;
            if let Some(existing) = store.get(key) {
                return existing == instance_id;
            }
            store.insert(key.to_string(), instance_id.to_string());
            true
        }

        /// Simulates renewing a distributed lock.
        ///
        /// This method checks if the lock exists and is owned by the specified
        /// instance. Unlike real Redis, it doesn't extend TTL as the mock
        /// doesn't implement expiration.
        ///
        /// # Parameters
        ///
        /// - `key` - The lock key
        /// - `instance_id` - The instance ID that should own the lock
        ///
        /// # Returns
        ///
        /// - `bool` - true if the lock exists and is owned by the specified
        ///   instance
        pub async fn renew_lock(&self, key: &str, instance_id: &str) -> bool {
            let store = self.store.read().await;
            if let Some(current) = store.get(key) {
                return current == instance_id;
            }
            false
        }

        /// Simulates releasing a distributed lock.
        ///
        /// This method removes the lock if it exists and is owned by the
        /// specified instance.
        ///
        /// # Parameters
        ///
        /// - `key` - The lock key
        /// - `instance_id` - The instance ID that should own the lock
        ///
        /// # Returns
        ///
        /// - `Result<()>` - Ok if the operation was successful
        pub async fn release_lock(&self, key: &str, instance_id: &str) -> Result<()> {
            let mut store = self.store.write().await;
            if let Some(current) = store.get(key) {
                if current == instance_id {
                    store.remove(key);
                }
            }
            Ok(())
        }

        /// Simulates posting a message to a queue.
        ///
        /// This method appends a message to the specified queue, creating the
        /// queue if it doesn't exist. Messages are separated by the '|'
        /// character.
        ///
        /// # Parameters
        ///
        /// - `queue` - The queue name
        /// - `message` - The message to post
        ///
        /// # Returns
        ///
        /// - `Result<()>` - Ok if the operation was successful
        pub async fn post_queue_message(&self, queue: &str, message: &str) -> Result<()> {
            let mut store = self.store.write().await;
            if let Some(existing) = store.get_mut(queue) {
                existing.push('|');
                existing.push_str(message);
            } else {
                store.insert(queue.to_string(), message.to_string());
            }
            Ok(())
        }

        /// Retrieves and removes the first message from a queue.
        ///
        /// This method gets the first message from the specified queue and
        /// removes it, simulating the behavior of Redis RPOPLPUSH command.
        ///
        /// # Parameters
        ///
        /// - `queue` - The queue name
        ///
        /// # Returns
        ///
        /// - `Result<Option<String>>` - The first message in the queue, or
        ///   None if empty
        pub async fn get_queue_message(&self, queue: &str) -> Result<Option<String>> {
            let mut store = self.store.write().await;
            if let Some(existing) = store.get_mut(queue) {
                if let Some(pos) = existing.find('|') {
                    let msg = existing[..pos].to_string();
                    *existing = existing[pos + 1..].to_string();
                    return Ok(Some(msg));
                }

                // If there's no separator but there is content, return the
                // entire content
                if !existing.is_empty() {
                    let msg = existing.clone();
                    *existing = String::new();
                    return Ok(Some(msg));
                }
            }
            Ok(None)
        }
    }
}

/// # Redis Client
///
/// The `RedisClient` is the primary interface for interacting with Redis in
/// the application.  It encapsulates all Redis-related functionality, providing
/// a unified API for various Redis operations including connection management,
/// leadership election, presence tracking, and messaging.
///
/// ## Key Features
///
/// * **Connection Management**: Handles Redis connection establishment, monitoring, and automatic reconnection
/// * **Leadership Election**: Distributed lock mechanism for leader election among service instances
/// * **Presence Tracking**: Service discovery and health monitoring capabilities
/// * **Messaging**: Pub/sub and queue-based messaging between service instances
/// * **Fallback Mechanism**: Automatic fallback to in-memory mock implementation when Redis is unavailable
///
/// ## Implementation Details
///
/// The client uses a modular architecture with specialized managers for
/// different Redis functionalities:
/// - `ConnectionManagerWrapper`: Manages Redis connections and reconnection logic
/// - `LeadershipManager`: Handles distributed locks for leader election
/// - `PresenceManager`: Manages service instance presence and discovery
/// - `MessagingManager`: Handles pub/sub and queue-based messaging
///
/// ## Thread Safety
///
/// The `RedisClient` is designed to be thread-safe, using `Arc` and
/// synchronization primitives to allow sharing across multiple tasks and
/// threads. Background tasks for connection monitoring and presence updates
/// are managed automatically.
///
/// ## Performance Considerations
///
/// To reduce mutex contention and improve performance:
/// - RwLock is used for structures with more reads than writes
///   (message_callbacks, locks)
/// - Consider connection pooling for high-throughput scenarios
/// - The mock implementation uses RwLock for better read concurrency
/// - For further optimization, consider implementing a custom connection
///   wrapper with
///   specialized read-only operations that don't require exclusive access
///
/// ## Usage Example
///
/// ```rust
/// let config = RedisConfig::default();
/// let client = RedisClient::new("my-service".to_string(), config).await?;
///
/// // Register a message handler
/// client.register_message_handler(Arc::new(|msg| {
///     println!("Received message: {}", msg);
/// })).await;
///
/// // Start background tasks
/// client.start_periodic_tasks();
///
/// // Use the client for various Redis operations
/// client.set_with_service_prefix("my-key", "my-value").await?;
/// ```
pub struct RedisClient {
    pub config: RedisConfig,
    pub service_name: String,
    pub instance_id: String,
    pub connection_manager: connection::ConnectionManagerWrapper,
    pub leadership_manager: leadership::LeadershipManager,
    pub presence_manager: presence::PresenceManager,
    pub messaging_manager: messaging::MessagingManager,

    pub message_callbacks: Arc<RwLock<Vec<MessageCallback>>>,
    pub listener_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    pub presence_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    pub connection_monitor_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    pub fallback: Option<mock::MockRedisClient>,

    pub locks: Arc<RwLock<std::collections::HashMap<String, (String, i64)>>>,
}

impl RedisClient {
    pub async fn new(service_name: String, config: RedisConfig) -> anyhow::Result<Self> {
        let client = Client::open(config.url.clone());
        let (connection_manager, fallback) = match client {
            Ok(c) => match connection::ConnectionManagerWrapper::new(&c).await {
                Ok(cm) => (cm, None),
                Err(e) => {
                    log::error!("<NewRedisClient> Failed to initialize real Redis connection: {}. Falling back to mock client.", e);
                    (
                        connection::ConnectionManagerWrapper {
                            connection: Arc::new(TokioMutex::new(
                                c.get_connection_manager().await?,
                            )),
                            pub_sub: Arc::new(TokioMutex::new(c.get_connection_manager().await?)),
                            state: Arc::new(AtomicBool::new(false)),
                        },
                        Some(mock::MockRedisClient::new()),
                    )
                }
            },
            Err(e) => {
                log::error!(
                    "<NewRedisClient> Failed to open Redis client: {}. Falling back to mock client.",
                    e
                );
                let dummy_client = Client::open("redis://127.0.0.1:1")?;
                let cm = connection::ConnectionManagerWrapper::new(&dummy_client).await?;
                (cm, Some(mock::MockRedisClient::new()))
            }
        };

        let instance_id =
            std::env::var("HOSTNAME").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

        let leadership_manager =
            leadership::LeadershipManager::new(instance_id.clone(), config.clone());
        let presence_manager =
            presence::PresenceManager::new(instance_id.clone(), config.key_prefix.clone());
        let messaging_manager = messaging::MessagingManager::new(config.key_prefix.clone());

        let client = RedisClient {
            config,
            service_name,
            instance_id,
            connection_manager,
            leadership_manager,
            presence_manager,
            messaging_manager,
            message_callbacks: Arc::new(RwLock::new(Vec::new())),
            listener_task: Arc::new(RwLock::new(None)),
            presence_task: Arc::new(RwLock::new(None)),
            connection_monitor_task: Arc::new(RwLock::new(None)),
            fallback,
            locks: Arc::new(RwLock::new(std::collections::HashMap::new())),
        };

        // Start the message listener
        client.start_message_listener().await?;
        Ok(client)
    }

    // ------------------------------
    // Message Listener and Handlers
    // ------------------------------
    pub async fn register_message_handler(&self, callback: Arc<dyn Fn(String) + Send + Sync>) {
        let mut callbacks = self.message_callbacks.write().await;
        let adapted_callback: MessageCallback = callback;
        callbacks.push(adapted_callback);
    }

    async fn start_message_listener(&self) -> anyhow::Result<()> {
        let instance_channel = format!(
            "{}::{}::msgchannel",
            self.config.key_prefix, self.instance_id
        );
        let broadcast_channel = format!("{}::msgchannel", self.config.key_prefix);

        log::info!(
            "<NewRedisClient> Starting message listener on channels: {} and {}",
            instance_channel,
            broadcast_channel
        );

        let _pub_sub_manager = self.connection_manager.pub_sub.clone();
        let callbacks = self.message_callbacks.clone();
        let config_url = self.config.url.clone();

        let instance_channel_clone = instance_channel.clone();
        let broadcast_channel_clone = broadcast_channel.clone();

        let listener = tokio::spawn(async move {
            // Obtain a pubsub connection
            let client = Client::open(config_url).expect("Failed to open client for listener");
            let pubsub_conn = match client.get_async_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    log::error!("<NewRedisClient> Failed to get pubsub connection: {}", e);
                    return;
                }
            };

            let mut pubsub = pubsub_conn.into_pubsub();
            if let Err(e) = pubsub
                .subscribe(&[&instance_channel_clone, &broadcast_channel_clone])
                .await
            {
                log::error!("<NewRedisClient> Failed to subscribe to channels: {}", e);
                return;
            }

            loop {
                let msg = pubsub.on_message().next().await;
                if let Some(msg) = msg {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        log::info!("<NewRedisClient> Received message: {}", payload);
                        let handlers = callbacks.read().await.clone();
                        for handler in handlers.iter() {
                            handler(payload.clone());
                        }
                    } else {
                        log::warn!("<NewRedisClient> Failed to decode message payload");
                    }
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        });

        // Store the listener task
        let listener_task_clone = self.listener_task.clone();
        tokio::spawn(async move {
            let mut listener_guard = listener_task_clone.write().await;
            *listener_guard = Some(listener);
        });

        Ok(())
    }

    // -----------------
    // Queue Operations
    // -----------------
    pub async fn post_queue_message(
        &self,
        message: &str,
        feature_name: Option<&str>,
    ) -> anyhow::Result<()> {
        let queue = match feature_name {
            Some(name) => format!("{}::{}::mqrecieved", self.config.key_prefix, name),
            None => format!("{}::mqrecieved", self.config.key_prefix),
        };

        if self.fallback.is_none() {
            let mut conn = self.connection_manager.connection.lock().await;
            let _: () = redis::cmd("RPUSH")
                .arg(&queue)
                .arg(message)
                .query_async(&mut *conn)
                .await?;
        } else if let Some(ref mock_client) = self.fallback {
            mock_client.post_queue_message(&queue, message).await?;
        }
        Ok(())
    }

    pub async fn get_queue_message(
        &self,
        feature_name: Option<&str>,
    ) -> anyhow::Result<Option<String>> {
        let source_queue = match feature_name {
            Some(name) => format!("{}::{}::mqrecieved", self.config.key_prefix, name),
            None => format!("{}::mqrecieved", self.config.key_prefix),
        };
        let destination_queue = match feature_name {
            Some(name) => format!("{}::{}::mqprocess", self.config.key_prefix, name),
            None => format!("{}::mqprocess", self.config.key_prefix),
        };
        if self.fallback.is_none() {
            let mut conn = self.connection_manager.connection.lock().await;
            let msg: Option<String> = redis::cmd("RPOPLPUSH")
                .arg(&source_queue)
                .arg(&destination_queue)
                .query_async(&mut *conn)
                .await?;
            Ok(msg)
        } else if let Some(ref mock_client) = self.fallback {
            let msg = mock_client.get_queue_message(&source_queue).await?;
            Ok(msg)
        } else {
            Ok(None)
        }
    }

    pub async fn mark_queue_message(
        &self,
        message: &str,
        success: bool,
        feature_name: Option<&str>,
    ) -> anyhow::Result<()> {
        let in_progress_queue = match feature_name {
            Some(name) => format!("{}::{}::mqprocess", self.config.key_prefix, name),
            None => format!("{}::mqprocess", self.config.key_prefix),
        };
        let incomplete_queue = match feature_name {
            Some(name) => format!("{}::{}::mqincomplete", self.config.key_prefix, name),
            None => format!("{}::mqincomplete", self.config.key_prefix),
        };
        if self.fallback.is_none() {
            let mut conn = self.connection_manager.connection.lock().await;
            if success {
                let _: i32 = redis::cmd("LREM")
                    .arg(&in_progress_queue)
                    .arg(0)
                    .arg(message)
                    .query_async(&mut *conn)
                    .await?;
            } else {
                let mut pipe = redis::pipe();
                pipe.lrem(&in_progress_queue, 0, message).ignore();
                pipe.rpush(&incomplete_queue, message).ignore();
                let _: () = pipe.query_async(&mut *conn).await?;
            }
        } else if let Some(ref _mock_client) = self.fallback {
            // For mock, we do nothing
            // placeholder for now
        }
        Ok(())
    }

    // -------------------------
    // Lock Lifecycle Functions
    // -------------------------
    pub async fn register_lock(&self, name: &str, ttl: i64) -> anyhow::Result<()> {
        let key = format!("{}::{}::lock", self.config.key_prefix, name);
        let mut locks = self.locks.write().await;
        locks.insert(name.to_string(), (key, ttl));
        Ok(())
    }

    pub async fn attempt_lock(&self, name: &str) -> anyhow::Result<bool> {
        let locks = self.locks.read().await;
        if let Some((lock_key, ttl)) = locks.get(name) {
            return Ok(self
                .leadership_manager
                .attempt_lock(self.connection_manager.connection.clone(), lock_key, *ttl)
                .await);
        }
        Ok(false)
    }

    pub async fn renew_lock(&self, name: &str) -> anyhow::Result<bool> {
        let locks = self.locks.read().await;
        if let Some((lock_key, ttl)) = locks.get(name) {
            let result = self
                .leadership_manager
                .renew_lock(
                    self.connection_manager.connection.clone(),
                    lock_key,
                    &self.instance_id,
                    *ttl,
                )
                .await?;
            return Ok(result);
        }
        Ok(false)
    }

    pub async fn check_and_renew_lock(&self, name: &str) -> anyhow::Result<(bool, bool)> {
        let had_lock = self.attempt_lock(name).await?;
        let now_has_lock = self.renew_lock(name).await?;
        Ok((now_has_lock, now_has_lock && !had_lock))
    }

    pub async fn release_lock(&self, name: &str) -> anyhow::Result<()> {
        let locks = self.locks.write().await;
        if let Some((lock_key, _)) = locks.get(name) {
            return self
                .leadership_manager
                .release_lock(
                    self.connection_manager.connection.clone(),
                    lock_key,
                    &self.instance_id,
                )
                .await;
        }
        Ok(())
    }

    pub async fn has_lock(&self, name: &str) -> anyhow::Result<bool> {
        let locks = self.locks.read().await;
        if let Some((lock_key, _)) = locks.get(name) {
            let result = self
                .leadership_manager
                .has_lock(
                    self.connection_manager.connection.clone(),
                    lock_key,
                    &self.instance_id,
                )
                .await?;
            return Ok(result);
        }
        Ok(false)
    }

    // --------------------------------------
    // Periodic Tasks and Connection Monitor
    // --------------------------------------
    pub fn start_periodic_tasks(&self) {
        // Spawn a periodic presence update task
        let connection = self.connection_manager.connection.clone();
        let key_prefix = self.config.key_prefix.clone();
        let instance_id = self.instance_id.clone();
        let presence_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                let key = format!("{}::{}::presence", key_prefix, instance_id);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let mut conn_guard = connection.lock().await;
                let _: redis::RedisResult<()> = redis::cmd("SETEX")
                    .arg(&key)
                    .arg(3)
                    .arg(now)
                    .query_async(&mut *conn_guard)
                    .await;
            }
        });

        // Store the presence task
        let presence_task_clone = self.presence_task.clone();
        tokio::spawn(async move {
            let mut presence_guard = presence_task_clone.write().await;
            *presence_guard = Some(presence_task);
        });

        // Start connection monitor task
        let connection_manager = self.connection_manager.clone();
        let config = self.config.clone();
        let monitor_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            loop {
                interval.tick().await;
                if !connection_manager.ping().await {
                    log::error!("<NewRedisClient> Redis connection lost, attempting reconnection");
                    let client = redis::Client::open(config.url.clone())
                        .expect("Failed to open client during reconnection");
                    connection_manager
                        .attempt_reconnection(&client, &config)
                        .await;
                }
            }
        });

        // Store the connection monitor task
        let connection_monitor_task_clone = self.connection_monitor_task.clone();
        tokio::spawn(async move {
            let mut guard = connection_monitor_task_clone.write().await;
            *guard = Some(monitor_task);
        });
    }

    pub fn start_periodic_tasks_compat(&mut self) {
        let connection_manager = self.connection_manager.clone();
        let presence_manager = self.presence_manager.clone();
        let _instance_id = self.instance_id.clone();
        let _config = self.config.clone();

        let presence_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                if let Err(e) = presence_manager
                    .update_presence(connection_manager.connection.clone())
                    .await
                {
                    log::error!("<NewRedisClient> Error updating presence: {}", e);
                }
            }
        });

        // Store the presence task
        let presence_task_clone = self.presence_task.clone();
        tokio::spawn(async move {
            let mut presence_task_guard = presence_task_clone.write().await;
            *presence_task_guard = Some(presence_task);
        });

        // Start connection monitor task
        self.start_connection_monitor();
    }

    pub fn start_connection_monitor(&self) -> tokio::task::JoinHandle<()> {
        // Create a closure that builds the monitoring task
        let connection_manager = self.connection_manager.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let client = redis::Client::open(config.url.clone()).unwrap();
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                if !connection_manager.ping().await {
                    connection_manager
                        .attempt_reconnection(&client, &config)
                        .await;
                }
            }
        })
    }

    pub fn stop_periodic_tasks(&mut self) -> anyhow::Result<()> {
        let rt = tokio::runtime::Handle::try_current()?;

        rt.block_on(async {
            if let Some(task) = self.presence_task.write().await.take() {
                task.abort();
            }

            if let Some(task) = self.connection_monitor_task.write().await.take() {
                task.abort();
            }

            if let Some(task) = self.listener_task.write().await.take() {
                task.abort();
            }
        });

        Ok(())
    }

    pub async fn check_connection(&mut self) -> anyhow::Result<()> {
        let is_connected = self.connection_manager.ping().await;
        self.connection_manager
            .state
            .store(is_connected, Ordering::SeqCst);

        if !is_connected {
            let client = Client::open(self.config.url.clone())?;
            self.connection_manager
                .attempt_reconnection(&client, &self.config)
                .await;
        }

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connection_manager.state.load(Ordering::SeqCst)
    }

    pub async fn set_with_service_prefix<V: redis::ToRedisArgs + Send + Sync>(
        &self,
        key: &str,
        value: V,
    ) -> anyhow::Result<()> {
        let prefixed_key = self.service_prefix(&[key]);
        let mut conn = self.connection_manager.connection.lock().await;
        conn.set::<_, _, ()>(&prefixed_key, value).await?;
        Ok(())
    }

    pub async fn get_with_service_prefix<V: redis::FromRedisValue + Send + Sync>(
        &self,
        key: &str,
    ) -> anyhow::Result<Option<V>> {
        let prefixed_key = self.service_prefix(&[key]);
        let mut conn = self.connection_manager.connection.lock().await;
        let result = conn.get(&prefixed_key).await?;
        Ok(result)
    }

    pub async fn get_queue_message_compat(
        &self,
        feature_name: Option<&str>,
    ) -> anyhow::Result<Option<String>> {
        let queue_key = match feature_name {
            Some(feature) => self.service_prefix(&["queue", feature]),
            None => self.service_prefix(&["queue"]),
        };
        let mut conn = self.connection_manager.connection.lock().await;
        let result: Option<String> = conn.lpop(&queue_key, None).await?;
        Ok(result)
    }

    pub async fn mark_queue_message_compat(
        &mut self,
        message: &str,
        success: bool,
        feature_name: Option<&str>,
    ) -> anyhow::Result<()> {
        let status = if success { "success" } else { "failure" };
        let queue_history_key = match feature_name {
            Some(feature) => self.service_prefix(&["queue", "history", feature, status]),
            None => self.service_prefix(&["queue", "history", status]),
        };

        let mut conn = self.connection_manager.connection.lock().await;
        conn.rpush::<_, _, ()>(&queue_history_key, message).await?;
        Ok(())
    }
}

impl Drop for RedisClient {
    fn drop(&mut self) {
        log::info!("<NewRedisClient> NewRedisClient is being dropped. Cleaning up tasks and releasing locks.");

        // We can't use await in drop, so we need to use blocking operations
        // or just ignore the errors for cleanup
        if let Ok(mut guard) = self.listener_task.try_write() {
            if let Some(task) = guard.take() {
                task.abort();
            }
        }

        if let Ok(mut guard) = self.presence_task.try_write() {
            if let Some(task) = guard.take() {
                task.abort();
            }
        }

        if let Ok(mut guard) = self.connection_monitor_task.try_write() {
            if let Some(task) = guard.take() {
                task.abort();
            }
        }

        // For locks, we can't use async operations in drop
        // We'll log this situation but can't properly clean up in a sync
        // context
        log::info!(
            "<RedisClient> Note: Locks will be automatically released when Redis TTL expires"
        );
    }
}

impl RedisClient {
    pub fn service_prefix(&self, keys: &[&str]) -> String {
        format!("{}::{}", self.config.key_prefix, keys.join("::"))
    }

    pub fn instance_prefix(&self, key: &str) -> String {
        self.service_prefix(&[&self.instance_id, key])
    }

    pub async fn presence_update(&mut self) -> anyhow::Result<()> {
        self.presence_manager
            .update_presence(self.connection_manager.connection.clone())
            .await
    }

    pub async fn register_lock_compat(&mut self, name: &str, ttl: i64) -> anyhow::Result<()> {
        let lock_key = self.service_prefix(&[name, "lock"]);
        let mut locks = self.locks.write().await;
        locks.insert(name.to_string(), (lock_key, ttl));
        Ok(())
    }

    pub async fn send_message_to_instance(
        &self,
        message: &str,
        target_instance_id: &str,
    ) -> anyhow::Result<()> {
        let channel = format!("{}::{}", self.config.key_prefix, target_instance_id);
        self.messaging_manager
            .publish_message(
                self.connection_manager.connection.clone(),
                &channel,
                message,
            )
            .await
    }

    pub async fn broadcast_message(&mut self, message: &str) -> anyhow::Result<()> {
        let broadcast_channel = self.service_prefix(&["msgchannel"]);
        self.messaging_manager
            .publish_message(
                self.connection_manager.connection.clone(),
                &broadcast_channel,
                message,
            )
            .await
    }

    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn get_service_name(&self) -> &str {
        &self.service_name
    }
}
