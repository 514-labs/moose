// This file implements a new, modular Redis client as per the proposal in scratch-pr-proposal.md.
use futures::FutureExt;
use futures::StreamExt;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time;

// Type alias for complex callback types
type MessageCallback =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

// -------------------------------
// Redis configuration
// -------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisConfig {
    #[serde(default = "RedisConfig::default_url")]
    pub url: String,
    #[serde(default = "RedisConfig::default_key_prefix")]
    pub key_prefix: String,
}

impl RedisConfig {
    pub fn default_url() -> String {
        "redis://127.0.0.1:6379".to_string()
    }

    pub fn default_key_prefix() -> String {
        "MS".to_string()
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        RedisConfig {
            url: RedisConfig::default_url(),
            key_prefix: RedisConfig::default_key_prefix(),
        }
    }
}

// -------------------------------
// Connection Module
// -------------------------------

pub mod connection {
    use super::*;

    pub enum ConnectionState {
        Connected,
        Disconnected,
        Reconnecting,
    }

    #[derive(Clone)]
    pub struct ConnectionManagerWrapper {
        pub connection: Arc<RwLock<ConnectionManager>>,
        pub pub_sub: Arc<RwLock<ConnectionManager>>,
        pub state: Arc<AtomicBool>, // true = connected
    }

    impl ConnectionManagerWrapper {
        pub async fn new(client: &Client) -> Result<Self, RedisError> {
            // Create connection manager for normal operations
            let conn = client.get_connection_manager().await?;
            let pub_sub_conn = client.get_connection_manager().await?;
            Ok(ConnectionManagerWrapper {
                connection: Arc::new(RwLock::new(conn)),
                pub_sub: Arc::new(RwLock::new(pub_sub_conn)),
                state: Arc::new(AtomicBool::new(true)),
            })
        }

        // A non-blocking ping with timeout and catch_unwind
        pub async fn ping(&self) -> bool {
            let mut conn_lock = self.connection.write().await;
            let timeout_future = time::timeout(
                Duration::from_secs(2),
                AssertUnwindSafe(async {
                    redis::cmd("PING")
                        .query_async::<_, String>(&mut *conn_lock)
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

        // Reconnection logic with exponential backoff
        pub async fn attempt_reconnection(&self, client: &Client, _config: &RedisConfig) {
            let mut backoff = 5;
            while !self.state.load(Ordering::SeqCst) {
                time::sleep(Duration::from_secs(backoff)).await;
                if let Ok(new_conn) = client.get_connection_manager().await {
                    let mut write_conn = self.connection.write().await;
                    *write_conn = new_conn;
                    self.state.store(true, Ordering::SeqCst);
                    break;
                } else {
                    backoff = std::cmp::min(backoff * 2, 60);
                }
            }
        }
    }
}

// -------------------------------
// Leadership Module
// -------------------------------

pub mod leadership {
    use super::*;
    use redis::Script;

    pub struct LeadershipManager {
        pub instance_id: String,
        pub redis_config: RedisConfig,
    }

    impl LeadershipManager {
        pub fn new(instance_id: String, config: RedisConfig) -> Self {
            LeadershipManager {
                instance_id,
                redis_config: config,
            }
        }

        // Attempt to acquire a leadership lock using a Lua script
        pub async fn attempt_lock(
            &self,
            conn: Arc<RwLock<ConnectionManager>>,
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
                let mut conn_guard = conn.write().await;
                script
                    .key(lock_key)
                    .arg(instance)
                    .arg(ttl)
                    .invoke_async::<_, i32>(&mut *conn_guard)
                    .await
            };
            matches!(result, Ok(val) if val == 1)
        }
    }
}

// -------------------------------
// Presence Module
// -------------------------------

pub mod presence {
    use super::*;
    use anyhow::Context;

    #[derive(Clone)]
    pub struct PresenceManager {
        pub instance_id: String,
        pub key_prefix: String,
    }

    impl PresenceManager {
        pub fn new(instance_id: String, key_prefix: String) -> Self {
            PresenceManager {
                instance_id,
                key_prefix,
            }
        }

        fn presence_key(&self) -> String {
            format!("{}::{}::presence", self.key_prefix, self.instance_id)
        }

        // Update presence using a TTL key
        pub async fn update_presence(
            &self,
            conn: Arc<RwLock<ConnectionManager>>,
        ) -> anyhow::Result<()> {
            let key = self.presence_key();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("Failed to get current time")?
                .as_secs();
            {
                let mut conn_guard = conn.write().await;
                let _: () = conn_guard.set_ex(&key, now, 10).await?;
            }
            Ok(())
        }
    }
}

// -------------------------------
// Messaging Module
// -------------------------------

pub mod messaging {
    use super::*;

    pub struct MessagingManager {
        pub key_prefix: String,
    }

    impl MessagingManager {
        pub fn new(key_prefix: String) -> Self {
            MessagingManager { key_prefix }
        }

        pub fn channel_for(&self, target: &str) -> String {
            format!("{}::{}::msgchannel", self.key_prefix, target)
        }

        // Publish a message to a channel
        pub async fn publish_message(
            &self,
            conn: Arc<RwLock<ConnectionManager>>,
            target: &str,
            message: &str,
        ) -> anyhow::Result<()> {
            let channel = self.channel_for(target);
            let mut conn_guard = conn.write().await;
            let _: () = conn_guard.publish(&channel, message).await?;
            Ok(())
        }
    }
}

// -------------------------------
// Additional Modules and Fallback
// -------------------------------

// New module for a basic Mock Redis Client fallback
pub mod mock {
    use anyhow::Result;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Default, Clone, Debug)]
    pub struct MockRedisClient {
        // Simple in-memory key-value store
        pub store: Arc<Mutex<HashMap<String, String>>>,
    }

    impl MockRedisClient {
        pub fn new() -> Self {
            Self {
                store: Arc::new(Mutex::new(HashMap::new())),
            }
        }

        pub async fn update_presence(&self, key: &str, value: u64) -> Result<()> {
            let mut store = self.store.lock().unwrap();
            store.insert(key.to_string(), value.to_string());
            Ok(())
        }

        pub async fn attempt_lock(&self, key: &str, instance_id: &str) -> bool {
            let mut store = self.store.lock().unwrap();
            if let Some(current) = store.get(key) {
                current == instance_id
            } else {
                store.insert(key.to_string(), instance_id.to_string());
                true
            }
        }

        pub async fn renew_lock(&self, key: &str, instance_id: &str) -> bool {
            let store = self.store.lock().unwrap();
            if let Some(current) = store.get(key) {
                return current == instance_id;
            }
            false
        }

        pub async fn release_lock(&self, key: &str, instance_id: &str) -> Result<()> {
            let mut store = self.store.lock().unwrap();
            if let Some(current) = store.get(key) {
                if current == instance_id {
                    store.remove(key);
                }
            }
            Ok(())
        }

        // Queue operations: simple Vec based queues stored in a HashMap
        pub async fn post_queue_message(&self, queue: &str, message: &str) -> Result<()> {
            let mut store = self.store.lock().unwrap();
            let entry = store.entry(queue.to_string()).or_default();
            *entry = format!("{};{}", entry, message);
            Ok(())
        }

        pub async fn get_queue_message(&self, queue: &str) -> Result<Option<String>> {
            let mut store = self.store.lock().unwrap();
            if let Some(existing) = store.get_mut(queue) {
                if let Some(pos) = existing.find('|') {
                    let msg = existing[..pos].to_string();
                    *existing = existing[pos + 1..].to_string();
                    return Ok(Some(msg));
                }
            }
            Ok(None)
        }
    }
}

// -------------------------------
// Extend the NewRedisClient struct with additional fields
// -------------------------------

pub struct NewRedisClient {
    pub config: RedisConfig,
    pub service_name: String,
    pub instance_id: String,
    pub connection_manager: connection::ConnectionManagerWrapper,
    pub leadership_manager: leadership::LeadershipManager,
    pub presence_manager: presence::PresenceManager,
    pub messaging_manager: messaging::MessagingManager,
    // New fields for additional features
    pub message_callbacks: Arc<RwLock<Vec<MessageCallback>>>,
    pub listener_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub presence_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub connection_monitor_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub fallback: Option<mock::MockRedisClient>,
    // For lock management, we maintain an in-memory map similar to the old locks HashMap
    pub locks: Arc<Mutex<std::collections::HashMap<String, (String, i64)>>>,
}

impl NewRedisClient {
    pub async fn new(service_name: String, config: RedisConfig) -> anyhow::Result<Self> {
        let client = Client::open(config.url.clone());
        let (connection_manager, fallback) = match client {
            Ok(c) => match connection::ConnectionManagerWrapper::new(&c).await {
                Ok(cm) => (cm, None),
                Err(e) => {
                    log::error!("Failed to initialize real Redis connection: {}. Falling back to mock client.", e);
                    (
                        connection::ConnectionManagerWrapper {
                            connection: Arc::new(RwLock::new(
                                Client::open("redis://127.0.0.1:1")?
                                    .get_connection_manager()
                                    .await?,
                            )),
                            pub_sub: Arc::new(RwLock::new(
                                Client::open(config.url.clone())?
                                    .get_connection_manager()
                                    .await?,
                            )),
                            state: Arc::new(AtomicBool::new(false)),
                        },
                        Some(mock::MockRedisClient::new()),
                    )
                }
            },
            Err(e) => {
                log::error!(
                    "Failed to open Redis client: {}. Falling back to mock client.",
                    e
                );
                let dummy_client = Client::open("redis://127.0.0.1:1")?;
                let cm = connection::ConnectionManagerWrapper::new(&dummy_client).await?;
                (cm, Some(mock::MockRedisClient::new()))
            }
        };

        // Determine instance_id, using HOSTNAME env or generate a UUID
        let instance_id =
            std::env::var("HOSTNAME").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

        let leadership_manager =
            leadership::LeadershipManager::new(instance_id.clone(), config.clone());
        let presence_manager =
            presence::PresenceManager::new(instance_id.clone(), config.key_prefix.clone());
        let messaging_manager = messaging::MessagingManager::new(config.key_prefix.clone());

        let client = NewRedisClient {
            config,
            service_name,
            instance_id,
            connection_manager,
            leadership_manager,
            presence_manager,
            messaging_manager,
            message_callbacks: Arc::new(RwLock::new(Vec::new())),
            listener_task: Arc::new(Mutex::new(None)),
            presence_task: Arc::new(Mutex::new(None)),
            connection_monitor_task: Arc::new(Mutex::new(None)),
            fallback,
            locks: Arc::new(Mutex::new(std::collections::HashMap::new())),
        };

        // Start the message listener
        client.start_message_listener().await?;
        Ok(client)
    }

    // -------------------------------
    // Message Listener and Handlers
    // -------------------------------

    pub async fn register_message_handler(&self, callback: MessageCallback) {
        let mut callbacks = self.message_callbacks.write().await;
        callbacks.push(callback);
    }

    async fn start_message_listener(&self) -> anyhow::Result<()> {
        // Define the instance and broadcast channels
        let instance_channel = format!(
            "{}::{}::msgchannel",
            self.config.key_prefix, self.instance_id
        );
        let broadcast_channel = format!("{}::msgchannel", self.config.key_prefix);

        log::info!(
            "Starting message listener on channels: {} and {}",
            instance_channel,
            broadcast_channel
        );

        // Clone necessary fields for the async task
        let _pub_sub_manager = self.connection_manager.pub_sub.clone();
        let callbacks = self.message_callbacks.clone();
        let config_url = self.config.url.clone();

        // Clone the channels for use in the task
        let instance_channel_clone = instance_channel.clone();
        let broadcast_channel_clone = broadcast_channel.clone();

        let listener = tokio::spawn(async move {
            // Obtain a pubsub connection
            let client = Client::open(config_url).expect("Failed to open client for listener");
            let pubsub_conn = match client.get_async_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    log::error!("Failed to get pubsub connection: {}", e);
                    return;
                }
            };

            let mut pubsub = pubsub_conn.into_pubsub();
            if let Err(e) = pubsub
                .subscribe(&[&instance_channel_clone, &broadcast_channel_clone])
                .await
            {
                log::error!("Failed to subscribe to channels: {}", e);
                return;
            }

            loop {
                let msg = pubsub.on_message().next().await;
                if let Some(msg) = msg {
                    if let Ok(payload) = msg.get_payload::<String>() {
                        log::info!("Received message: {}", payload);
                        let handlers = callbacks.read().await.clone();
                        for handler in handlers.iter() {
                            handler(payload.clone()).await;
                        }
                    } else {
                        log::warn!("Failed to decode message payload");
                    }
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        });

        // Save the listener task handle
        let mut listener_guard = self.listener_task.lock().unwrap();
        *listener_guard = Some(listener);
        drop(listener_guard);

        Ok(())
    }

    // -------------------------------
    // Queue Operations
    // -------------------------------

    pub async fn post_queue_message(
        &self,
        message: &str,
        feature_name: Option<&str>,
    ) -> anyhow::Result<()> {
        let queue = match feature_name {
            Some(name) => format!("{}::{}::mqrecieved", self.config.key_prefix, name),
            None => format!("{}::mqrecieved", self.config.key_prefix),
        };
        // Use real connection if available, otherwise fallback
        if self.fallback.is_none() {
            let mut conn = self.connection_manager.connection.write().await;
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
            let mut conn = self.connection_manager.connection.write().await;
            // Using RPOPLPUSH
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
            let mut conn = self.connection_manager.connection.write().await;
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
        }
        Ok(())
    }

    // -------------------------------
    // Lock Lifecycle Functions
    // -------------------------------

    pub async fn register_lock(&self, name: &str, ttl: i64) -> anyhow::Result<()> {
        let key = format!("{}::{}::lock", self.config.key_prefix, name);
        let mut locks = self.locks.lock().unwrap();
        locks.insert(name.to_string(), (key, ttl));
        Ok(())
    }

    pub async fn renew_lock(&self, name: &str) -> anyhow::Result<bool> {
        // Get lock information
        let key;
        let ttl;
        {
            let locks = self.locks.lock().unwrap();
            if let Some((lock_key, lock_ttl)) = locks.get(name) {
                key = lock_key.clone();
                ttl = *lock_ttl;
            } else {
                return Ok(false);
            }
        }

        // Attempt to renew the lock
        if self.fallback.is_none() {
            let mut conn = self.connection_manager.connection.write().await;
            let script = redis::Script::new(
                "if redis.call('GET', KEYS[1]) == ARGV[1] then return redis.call('EXPIRE', KEYS[1], ARGV[2]) else return 0 end",
            );
            let result: i32 = script
                .key(&key)
                .arg(&self.instance_id)
                .arg(ttl)
                .invoke_async(&mut *conn)
                .await?;
            Ok(result == 1)
        } else if let Some(ref mock_client) = self.fallback {
            let result = mock_client.renew_lock(&key, &self.instance_id).await;
            Ok(result)
        } else {
            Ok(false)
        }
    }

    pub async fn release_lock(&self, name: &str) -> anyhow::Result<()> {
        let mut locks = self.locks.lock().unwrap();
        if let Some((key, _ttl)) = locks.get(name) {
            let _: anyhow::Result<()> = if self.fallback.is_none() {
                let rt = tokio::runtime::Handle::try_current();
                if let Ok(handle) = rt {
                    handle.block_on(async {
                        let mut conn = self.connection_manager.connection.write().await;
                        let script = redis::Script::new(
                            "if redis.call('GET', KEYS[1]) == ARGV[1] then\n return redis.call('DEL', KEYS[1])\nelse\n return 0\nend"
                        );
                        let _: () = script.key(key).arg(&self.instance_id).invoke_async(&mut *conn).await.unwrap_or(());
                    });
                }
                Ok(())
            } else if let Some(ref mock_client) = self.fallback {
                let _ =
                    futures::executor::block_on(mock_client.release_lock(key, &self.instance_id));
                Ok(())
            } else {
                Ok(())
            };
            locks.remove(name);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Lock not registered"))
        }
    }

    pub async fn check_and_renew_lock(&self, name: &str) -> anyhow::Result<(bool, bool)> {
        // Get lock information
        let key;
        {
            let locks = self.locks.lock().unwrap();
            if let Some((lock_key, _)) = locks.get(name) {
                key = lock_key.clone();
            } else {
                return Ok((false, false));
            }
        }

        // Check if we have the lock and renew it
        if self.fallback.is_none() {
            let mut conn = self.connection_manager.connection.write().await;
            let script = redis::Script::new(
                r#"
                local current = redis.call('GET', KEYS[1])
                if current == ARGV[1] then
                    redis.call('EXPIRE', KEYS[1], ARGV[2])
                    return {1, 1}
                elseif current then
                    return {0, 1}
                else
                    return {0, 0}
                end
                "#,
            );
            let result: Vec<i32> = script
                .key(&key)
                .arg(&self.instance_id)
                .arg(10) // Default TTL for now
                .invoke_async(&mut *conn)
                .await?;

            if result.len() >= 2 {
                Ok((result[0] == 1, result[1] == 1))
            } else {
                Err(anyhow::anyhow!("Unexpected response from Redis"))
            }
        } else if let Some(ref mock_client) = self.fallback {
            let has_lock = mock_client.attempt_lock(&key, &self.instance_id).await;
            Ok((has_lock, true))
        } else {
            Ok((false, false))
        }
    }

    // -------------------------------
    // Periodic Tasks and Connection Monitor
    // -------------------------------

    pub fn start_periodic_tasks(&self) {
        // Spawn a periodic presence update task
        let _presence_manager = self.presence_manager.clone();
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
                let mut conn_guard = connection.write().await;
                let _: redis::RedisResult<()> = redis::cmd("SETEX")
                    .arg(&key)
                    .arg(3)
                    .arg(now)
                    .query_async(&mut *conn_guard)
                    .await;
            }
        });

        // Save the presence task handle
        let mut presence_guard = self.presence_task.lock().unwrap();
        *presence_guard = Some(presence_task);
        drop(presence_guard);

        // Spawn a connection monitor task
        let connection_monitor = self.connection_manager.clone();
        let config = self.config.clone();
        let monitor_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop {
                interval.tick().await;
                if !connection_monitor.ping().await {
                    log::error!("Redis connection lost, attempting reconnection");
                    let client = Client::open(config.url.clone())
                        .expect("Failed to open client during reconnection");
                    connection_monitor
                        .attempt_reconnection(&client, &config)
                        .await;
                }
            }
        });

        // Save the connection monitor task handle
        let mut monitor_guard = self.connection_monitor_task.lock().unwrap();
        *monitor_guard = Some(monitor_task);
        drop(monitor_guard);
    }

    // -------------------------------
    // Cleanup Logic in Drop
    // -------------------------------
}

impl Drop for NewRedisClient {
    fn drop(&mut self) {
        log::info!("NewRedisClient is being dropped. Cleaning up tasks and releasing locks.");
        if let Some(task) = self.listener_task.lock().unwrap().take() {
            task.abort();
        }
        if let Some(task) = self.presence_task.lock().unwrap().take() {
            task.abort();
        }
        if let Some(task) = self.connection_monitor_task.lock().unwrap().take() {
            task.abort();
        }
        // Release all registered locks
        let locks = self.locks.lock().unwrap();
        for (name, (key, _ttl)) in locks.iter() {
            let _: anyhow::Result<()> = if self.fallback.is_none() {
                let rt = tokio::runtime::Handle::try_current();
                if let Ok(handle) = rt {
                    handle.block_on(async {
                        let mut conn = self.connection_manager.connection.write().await;
                        let script = redis::Script::new(
                            "if redis.call('GET', KEYS[1]) == ARGV[1] then\n return redis.call('DEL', KEYS[1])\nelse\n return 0\nend"
                        );
                        let _: () = script.key(key).arg(&self.instance_id).invoke_async(&mut *conn).await.unwrap_or(());
                    });
                }
                Ok(())
            } else if let Some(ref mock_client) = self.fallback {
                let _ =
                    futures::executor::block_on(mock_client.release_lock(key, &self.instance_id));
                Ok(())
            } else {
                Ok(())
            };
            log::info!("Released lock {}", name);
        }
    }
}
