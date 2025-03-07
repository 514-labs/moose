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
use futures::StreamExt;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

// Import the modules
use super::connection::ConnectionManagerWrapper;
use super::leadership::LeadershipManager;
use super::messaging::MessagingManager;
use super::mock::MockRedisClient;
use super::presence::PresenceManager;

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
///     url: "redis://my-redis-server:6379".to_string(),
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
    pub connection_manager: ConnectionManagerWrapper,
    pub leadership_manager: LeadershipManager,
    pub presence_manager: PresenceManager,
    pub messaging_manager: MessagingManager,

    pub message_callbacks: Arc<RwLock<Vec<MessageCallback>>>,
    pub listener_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    pub presence_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    pub connection_monitor_task: Arc<RwLock<Option<JoinHandle<()>>>>,
    pub fallback: Option<MockRedisClient>,

    pub locks: Arc<RwLock<std::collections::HashMap<String, (String, i64)>>>,
}

impl RedisClient {
    pub async fn new(service_name: String, config: RedisConfig) -> anyhow::Result<Self> {
        let instance_id = uuid::Uuid::new_v4().to_string();

        // Create Redis client
        let client_result = Client::open(config.url.clone());
        let (connection_manager, fallback) = match client_result {
            Ok(c) => match ConnectionManagerWrapper::new(&c).await {
                Ok(cm) => (cm, None),
                Err(_) => {
                    // If we can't connect to Redis, use the mock client
                    let dummy_client = Client::open("redis://localhost:6379").unwrap();
                    let cm = ConnectionManagerWrapper::new(&dummy_client).await?;
                    (cm, Some(MockRedisClient::new()))
                }
            },
            Err(_) => {
                // If we can't even create a client, use the mock client
                let dummy_client = Client::open("redis://localhost:6379").unwrap();
                let cm = ConnectionManagerWrapper::new(&dummy_client).await?;
                (cm, Some(MockRedisClient::new()))
            }
        };

        // Create managers
        let leadership_manager = LeadershipManager::new(instance_id.clone(), config.clone());
        let presence_manager = PresenceManager::new(instance_id.clone(), config.key_prefix.clone());
        let messaging_manager = MessagingManager::new(config.key_prefix.clone());

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
