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
use tokio::sync::Mutex;
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

/// State container for RedisClient that encapsulates all synchronized state.
///
/// This struct centralizes the management of state that requires synchronization,
/// making it easier to reason about thread safety and concurrency. It uses
/// appropriate synchronization primitives (RwLock) for each type of state:
///
/// - RwLock for structures that have more reads than writes (callbacks, tasks, locks)
/// - Arc for shared ownership across tasks
///
/// By grouping these synchronized components together, we can more easily
/// ensure proper locking discipline and avoid potential deadlocks.
pub struct RedisClientState {
    /// Registered message handlers that will be called when messages are received
    pub message_callbacks: Arc<RwLock<Vec<MessageCallback>>>,

    /// Task handle for the message listener background task
    pub listener_task: Arc<RwLock<Option<JoinHandle<()>>>>,

    /// Task handle for the presence update background task
    pub presence_task: Arc<RwLock<Option<JoinHandle<()>>>>,

    /// Task handle for the connection monitor background task
    pub connection_monitor_task: Arc<RwLock<Option<JoinHandle<()>>>>,

    /// Map of registered locks with their keys and TTLs
    pub locks: Arc<RwLock<std::collections::HashMap<String, (String, i64)>>>,

    /// Fallback mock client used when Redis is unavailable
    pub fallback: Option<MockRedisClient>,
}

impl RedisClientState {
    /// Creates a new RedisClientState with empty/default values
    pub fn new(fallback: Option<MockRedisClient>) -> Self {
        RedisClientState {
            message_callbacks: Arc::new(RwLock::new(Vec::new())),
            listener_task: Arc::new(RwLock::new(None)),
            presence_task: Arc::new(RwLock::new(None)),
            connection_monitor_task: Arc::new(RwLock::new(None)),
            locks: Arc::new(RwLock::new(std::collections::HashMap::new())),
            fallback,
        }
    }
}

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
#[derive(Clone)]
pub struct RedisClient {
    pub config: RedisConfig,
    pub service_name: String,
    pub instance_id: String,
    pub connection_manager: ConnectionManagerWrapper,
    pub leadership_manager: LeadershipManager,
    pub presence_manager: PresenceManager,
    pub messaging_manager: MessagingManager,
    pub state: Arc<RedisClientState>,
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

        let state = Arc::new(RedisClientState::new(fallback));

        let client = RedisClient {
            config,
            service_name,
            instance_id,
            connection_manager,
            leadership_manager,
            presence_manager,
            messaging_manager,
            state,
        };

        // Start the message listener
        client.start_message_listener().await?;
        Ok(client)
    }

    // ------------------------------
    // Message Listener and Handlers
    // ------------------------------
    pub async fn register_message_handler(&self, callback: Arc<dyn Fn(String) + Send + Sync>) {
        let mut callbacks = self.state.message_callbacks.write().await;
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
        let callbacks = self.state.message_callbacks.clone();
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
        let listener_task_clone = self.state.listener_task.clone();
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
        let _messaging_manager = self.messaging_manager.clone();
        let message = message.to_string();

        self.connection_manager
            .execute(move |conn| {
                let feature_name_clone = feature_name.map(|s| s.to_string());
                Box::pin(async move {
                    let queue_key = match &feature_name_clone {
                        Some(feature) => format!("{}::{}::queue", feature, "messages"),
                        None => "messages::queue".to_string(),
                    };

                    let processing_queue_key = match &feature_name_clone {
                        Some(feature) => format!("{}::{}::processing", feature, "messages"),
                        None => "messages::processing".to_string(),
                    };

                    let failed_queue_key = match &feature_name_clone {
                        Some(feature) => format!("{}::{}::failed", feature, "messages"),
                        None => "messages::failed".to_string(),
                    };

                    // Create a pipeline to execute multiple commands
                    let mut pipe = redis::pipe();
                    pipe.atomic()
                        .cmd("SADD")
                        .arg("message_queues")
                        .arg(&queue_key)
                        .cmd("SADD")
                        .arg("message_processing_queues")
                        .arg(&processing_queue_key)
                        .cmd("SADD")
                        .arg("message_failed_queues")
                        .arg(&failed_queue_key)
                        .lpush(&queue_key, &message);

                    // Execute the pipeline
                    pipe.query_async::<_, ()>(conn).await
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to post message to queue: {}", e))
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
        if self.state.fallback.is_none() {
            let source = source_queue.clone();
            let dest = destination_queue.clone();
            let result = self
                .connection_manager
                .execute(|conn| {
                    Box::pin(async move {
                        redis::cmd("RPOPLPUSH")
                            .arg(&source)
                            .arg(&dest)
                            .query_async(conn)
                            .await
                    })
                })
                .await?;
            Ok(result)
        } else if let Some(ref mock_client) = self.state.fallback {
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
        let _messaging_manager = self.messaging_manager.clone();
        let message = message.to_string();

        self.connection_manager
            .execute(move |conn| {
                let feature_name_clone = feature_name.map(|s| s.to_string());
                Box::pin(async move {
                    let processing_queue_key = match &feature_name_clone {
                        Some(feature) => format!("{}::{}::processing", feature, "messages"),
                        None => "messages::processing".to_string(),
                    };

                    let target_queue_key = if success {
                        match &feature_name_clone {
                            Some(feature) => format!("{}::{}::completed", feature, "messages"),
                            None => "messages::completed".to_string(),
                        }
                    } else {
                        match &feature_name_clone {
                            Some(feature) => format!("{}::{}::failed", feature, "messages"),
                            None => "messages::failed".to_string(),
                        }
                    };

                    // Create a pipeline to execute multiple commands
                    let mut pipe = redis::pipe();
                    pipe.atomic()
                        .lrem(&processing_queue_key, 0, &message)
                        .lpush(&target_queue_key, &message);

                    // Execute the pipeline
                    pipe.query_async::<_, ()>(conn).await
                })
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to mark queue message: {}", e))
    }

    // -------------------------
    // Lock Lifecycle Functions
    // -------------------------
    pub async fn register_lock(&self, name: &str, ttl: i64) -> anyhow::Result<()> {
        let key = format!("{}::{}::lock", self.config.key_prefix, name);
        let mut locks = self.state.locks.write().await;
        locks.insert(name.to_string(), (key, ttl));
        Ok(())
    }

    pub async fn attempt_lock(&self, name: &str) -> anyhow::Result<bool> {
        let lock_key_and_ttl = {
            let locks = self.state.locks.read().await;
            locks.get(name).map(|(k, ttl)| (k.clone(), *ttl))
        };

        if let Some((lock_key, ttl)) = lock_key_and_ttl {
            let instance_id = self.instance_id.clone();
            let leadership_manager = self.leadership_manager.clone();
            return self
                .connection_manager
                .execute(|conn| {
                    Box::pin(async move {
                        leadership_manager
                            .attempt_lock_with_conn(conn, &lock_key, &instance_id, ttl)
                            .await
                    })
                })
                .await
                .map_err(Into::into);
        }
        Ok(false)
    }

    pub async fn renew_lock(&self, name: &str) -> anyhow::Result<bool> {
        let lock_key_and_ttl = {
            let locks = self.state.locks.read().await;
            locks.get(name).map(|(k, ttl)| (k.clone(), *ttl))
        };

        if let Some((lock_key, ttl)) = lock_key_and_ttl {
            let instance_id = self.instance_id.clone();
            let leadership_manager = self.leadership_manager.clone();
            return self
                .connection_manager
                .execute(|conn| {
                    Box::pin(async move {
                        leadership_manager
                            .renew_lock_with_conn(conn, &lock_key, &instance_id, ttl)
                            .await
                    })
                })
                .await
                .map_err(Into::into);
        }
        Ok(false)
    }

    pub async fn check_and_renew_lock(&self, name: &str) -> anyhow::Result<(bool, bool)> {
        let had_lock = self.attempt_lock(name).await?;
        let now_has_lock = self.renew_lock(name).await?;
        Ok((now_has_lock, now_has_lock && !had_lock))
    }

    pub async fn release_lock(&self, name: &str) -> anyhow::Result<()> {
        let lock_key = {
            let locks = self.state.locks.read().await;
            locks.get(name).map(|(k, _)| k.clone())
        };

        if let Some(lock_key) = lock_key {
            let instance_id = self.instance_id.clone();
            let leadership_manager = self.leadership_manager.clone();
            return self
                .connection_manager
                .execute(|conn| {
                    Box::pin(async move {
                        leadership_manager
                            .release_lock_with_conn(conn, &lock_key, &instance_id)
                            .await
                    })
                })
                .await
                .map_err(Into::into);
        }
        Ok(())
    }

    pub async fn has_lock(&self, name: &str) -> anyhow::Result<bool> {
        let lock_key_and_ttl = {
            let locks = self.state.locks.read().await;
            locks.get(name).map(|(k, _)| k.clone())
        };

        if let Some(lock_key) = lock_key_and_ttl {
            let instance_id = self.instance_id.clone();
            let leadership_manager = self.leadership_manager.clone();
            let result = self
                .connection_manager
                .execute(|conn| {
                    Box::pin(async move {
                        leadership_manager
                            .has_lock_with_conn(conn, &lock_key, &instance_id)
                            .await
                    })
                })
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
        let connection_manager = self.connection_manager.clone();
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

                let key_clone = key.clone();
                let now_clone = now;
                let _: redis::RedisResult<()> = connection_manager
                    .execute(|conn| {
                        Box::pin(async move {
                            redis::cmd("SETEX")
                                .arg(&key_clone)
                                .arg(3)
                                .arg(now_clone)
                                .query_async::<_, ()>(conn)
                                .await
                        })
                    })
                    .await;
            }
        });

        // Store the presence task
        let presence_task_clone = self.state.presence_task.clone();
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
        let connection_monitor_task_clone = self.state.connection_monitor_task.clone();
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
                let presence_manager_clone = presence_manager.clone();
                if let Err(e) = connection_manager
                    .execute(|conn| {
                        Box::pin(async move {
                            presence_manager_clone.update_presence_with_conn(conn).await
                        })
                    })
                    .await
                    .map_err(|e| anyhow::anyhow!("Redis error: {}", e))
                {
                    log::error!("<NewRedisClient> Error updating presence: {}", e);
                }
            }
        });

        // Store the presence task
        let presence_task_clone = self.state.presence_task.clone();
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
            if let Some(task) = self.state.presence_task.write().await.take() {
                task.abort();
            }

            if let Some(task) = self.state.connection_monitor_task.write().await.take() {
                task.abort();
            }

            if let Some(task) = self.state.listener_task.write().await.take() {
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

    pub async fn set_with_service_prefix<V>(&self, key: &str, value: V) -> anyhow::Result<()>
    where
        V: redis::ToRedisArgs + Send + Sync + Clone + 'static,
    {
        let prefixed_key = self.service_prefix(&[key]);
        let value_clone = value.clone();
        self.connection_manager
            .execute(|conn| {
                Box::pin(async move { conn.set::<_, _, ()>(&prefixed_key, value_clone).await })
            })
            .await
            .map_err(Into::into)
    }

    pub async fn get_with_service_prefix<V: redis::FromRedisValue + Send + Sync>(
        &self,
        key: &str,
    ) -> anyhow::Result<Option<V>> {
        let prefixed_key = self.service_prefix(&[key]);
        let result = self
            .connection_manager
            .execute(|conn| Box::pin(async move { conn.get(&prefixed_key).await }))
            .await;
        self.map_redis_error(result)
    }

    pub async fn get_queue_message_compat(
        &self,
        feature_name: Option<&str>,
    ) -> anyhow::Result<Option<String>> {
        let queue_key = match feature_name {
            Some(feature) => self.service_prefix(&["queue", feature]),
            None => self.service_prefix(&["queue"]),
        };
        let queue_key_clone = queue_key.clone();
        let result = self
            .connection_manager
            .execute(|conn| Box::pin(async move { conn.lpop(&queue_key_clone, None).await }))
            .await?;
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
        let queue_key_clone = queue_history_key.clone();
        let message_clone = message.to_string();
        self.connection_manager
            .execute(|conn| {
                Box::pin(async move {
                    conn.rpush::<_, _, ()>(&queue_key_clone, message_clone)
                        .await
                })
            })
            .await?;
        Ok(())
    }

    // Helper method for consistent error mapping
    fn map_redis_error<T>(&self, result: redis::RedisResult<T>) -> anyhow::Result<T> {
        result.map_err(|e| anyhow::anyhow!("Redis error: {}", e))
    }

    /// Creates a thread-safe wrapper for this RedisClient
    ///
    /// This method wraps the RedisClient in an Arc<Mutex<>> to make it
    /// safely shareable across threads and tasks. This is the recommended
    /// way to share a RedisClient instance in a multi-threaded environment.
    ///
    /// # Returns
    ///
    /// An Arc<Mutex<RedisClient>> that can be cloned and shared across tasks
    pub fn into_thread_safe(self) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(self))
    }

    /// Creates a new thread-safe RedisClient
    ///
    /// This is a convenience method that creates a new RedisClient and
    /// immediately wraps it in an Arc<Mutex<>> for thread-safe usage.
    ///
    /// # Arguments
    ///
    /// * `service_name` - The name of the service using this client
    /// * `config` - Redis configuration
    ///
    /// # Returns
    ///
    /// An Arc<Mutex<RedisClient>> that can be cloned and shared across tasks
    ///
    /// # Errors
    ///
    /// Returns an error if the RedisClient creation fails
    pub async fn new_thread_safe(
        service_name: String,
        config: RedisConfig,
    ) -> anyhow::Result<Arc<Mutex<Self>>> {
        let client = Self::new(service_name, config).await?;
        Ok(client.into_thread_safe())
    }
}

impl Drop for RedisClient {
    fn drop(&mut self) {
        log::info!("<NewRedisClient> NewRedisClient is being dropped. Cleaning up tasks and releasing locks.");

        // We can't use await in drop, so we need to use blocking operations
        // or just ignore the errors for cleanup
        if let Ok(mut guard) = self.state.listener_task.try_write() {
            if let Some(task) = guard.take() {
                task.abort();
            }
        }

        if let Ok(mut guard) = self.state.presence_task.try_write() {
            if let Some(task) = guard.take() {
                task.abort();
            }
        }

        if let Ok(mut guard) = self.state.connection_monitor_task.try_write() {
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
        let presence_manager = self.presence_manager.clone();
        self.connection_manager
            .execute(|conn| {
                Box::pin(async move { presence_manager.update_presence_with_conn(conn).await })
            })
            .await
            .map_err(Into::into)
    }

    pub async fn register_lock_compat(&mut self, name: &str, ttl: i64) -> anyhow::Result<()> {
        let lock_key = self.service_prefix(&[name, "lock"]);
        let mut locks = self.state.locks.write().await;
        locks.insert(name.to_string(), (lock_key, ttl));
        Ok(())
    }

    pub async fn send_message_to_instance(
        &self,
        message: &str,
        target_instance_id: &str,
    ) -> anyhow::Result<()> {
        let channel = format!("{}::{}", self.config.key_prefix, target_instance_id);
        let messaging_manager = self.messaging_manager.clone();
        let channel_clone = channel.clone();
        let message_clone = message.to_string();
        self.connection_manager
            .execute(|conn| {
                Box::pin(async move {
                    messaging_manager
                        .publish_message_with_conn(conn, &channel_clone, &message_clone)
                        .await
                })
            })
            .await
            .map_err(Into::into)
    }

    pub async fn broadcast_message(&mut self, message: &str) -> anyhow::Result<()> {
        let broadcast_channel = self.service_prefix(&["msgchannel"]);
        let messaging_manager = self.messaging_manager.clone();
        let channel_clone = broadcast_channel.clone();
        let message_clone = message.to_string();
        self.connection_manager
            .execute(|conn| {
                Box::pin(async move {
                    messaging_manager
                        .publish_message_with_conn(conn, &channel_clone, &message_clone)
                        .await
                })
            })
            .await
            .map_err(Into::into)
    }

    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn get_service_name(&self) -> &str {
        &self.service_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_redis_client_clone() {
        // Create a Redis client with default config
        let client = RedisClient::new("test-service".to_string(), RedisConfig::default())
            .await
            .unwrap();

        // Clone the client
        let client_clone = client.clone();

        // Verify that both clients have the same instance ID
        assert_eq!(client.instance_id, client_clone.instance_id);

        // Verify that both clients share the same state
        let key = "test-lock";
        let ttl = 30;

        // Register a lock using the original client
        client.register_lock(key, ttl).await.unwrap();

        // Verify the lock exists using the cloned client
        let lock_exists = {
            let locks = client_clone.state.locks.read().await;
            locks.contains_key(key)
        };

        assert!(
            lock_exists,
            "Lock should exist in the cloned client's state"
        );
    }

    #[tokio::test]
    async fn test_redis_client_shared_across_tasks() {
        // Create a Redis client with default config
        let client = RedisClient::new("test-service".to_string(), RedisConfig::default())
            .await
            .unwrap();

        // Wrap in Arc<Mutex<>> to simulate how it's used in the application
        let client = Arc::new(Mutex::new(client));

        // Spawn tasks that use the client
        let mut handles = vec![];

        for i in 0..5 {
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                let client = client_clone.lock().await;
                client
                    .register_lock(&format!("test-lock-{}", i), 30)
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all locks were registered
        let client = client.lock().await;
        let locks = client.state.locks.read().await;

        for i in 0..5 {
            let key = format!("test-lock-{}", i);
            assert!(locks.contains_key(&key), "Lock {} should exist", key);
        }
    }
}
