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

        pub async fn renew_lock(
            &self,
            conn: Arc<RwLock<ConnectionManager>>,
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
                .invoke_async::<_, i32>(&mut *conn.write().await)
                .await?;
            Ok(result == 1)
        }

        pub async fn release_lock(
            &self,
            conn: Arc<RwLock<ConnectionManager>>,
            lock_key: &str,
            instance_id: &str,
        ) -> anyhow::Result<()> {
            let script = redis::Script::new(
                "if redis.call('GET', KEYS[1]) == ARGV[1] then\n return redis.call('DEL', KEYS[1])\nelse\n return 0\nend"
            );
            let _result: () = script
                .key(lock_key)
                .arg(instance_id)
                .invoke_async(&mut *conn.write().await)
                .await?;
            Ok(())
        }

        pub async fn has_lock(
            &self,
            conn: Arc<RwLock<ConnectionManager>>,
            lock_key: &str,
            instance_id: &str,
        ) -> anyhow::Result<bool> {
            let script = redis::Script::new(
                "if redis.call('GET', KEYS[1]) == ARGV[1] then return 1 else return 0 end",
            );
            let result: i32 = script
                .key(lock_key)
                .arg(instance_id)
                .invoke_async::<_, i32>(&mut *conn.write().await)
                .await?;
            Ok(result == 1)
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
    pub locks: Arc<tokio::sync::Mutex<std::collections::HashMap<String, (String, i64)>>>,
}

impl NewRedisClient {
    pub async fn new(service_name: String, config: RedisConfig) -> anyhow::Result<Self> {
        let client = Client::open(config.url.clone());
        let (connection_manager, fallback) = match client {
            Ok(c) => match connection::ConnectionManagerWrapper::new(&c).await {
                Ok(cm) => (cm, None),
                Err(e) => {
                    log::error!("<NewRedisClient> Failed to initialize real Redis connection: {}. Falling back to mock client.", e);
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
                    "<NewRedisClient> Failed to open Redis client: {}. Falling back to mock client.",
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
            locks: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
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
            "<NewRedisClient> Starting message listener on channels: {} and {}",
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
                            handler(payload.clone()).await;
                        }
                    } else {
                        log::warn!("<NewRedisClient> Failed to decode message payload");
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
        let mut locks = self.locks.lock().await;
        locks.insert(name.to_string(), (key, ttl));
        Ok(())
    }

    pub async fn attempt_lock(&self, name: &str) -> anyhow::Result<bool> {
        let locks = self.locks.lock().await;
        if let Some((lock_key, ttl)) = locks.get(name) {
            return Ok(self
                .leadership_manager
                .attempt_lock(self.connection_manager.connection.clone(), lock_key, *ttl)
                .await);
        }
        Ok(false)
    }

    pub async fn renew_lock(&self, name: &str) -> anyhow::Result<bool> {
        // Map to the existing check_and_renew_lock
        let locks = self.locks.lock().await;
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
        // First check if we already have the lock
        let had_lock = self.attempt_lock(name).await?;

        // Then attempt to renew it
        let now_has_lock = self.renew_lock(name).await?;

        // Return if we have the lock and if this is a new acquisition
        Ok((now_has_lock, now_has_lock && !had_lock))
    }

    pub async fn release_lock(&self, name: &str) -> anyhow::Result<()> {
        let locks = self.locks.lock().await;
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
        let locks = self.locks.lock().await;
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

    // -------------------------------
    // Periodic Tasks and Connection Monitor
    // -------------------------------

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
        let connection_manager = self.connection_manager.clone();
        let config = self.config.clone();
        if let Ok(mut guard) = self.connection_monitor_task.lock() {
            *guard = Some(tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
                loop {
                    interval.tick().await;
                    if !connection_manager.ping().await {
                        log::error!(
                            "<NewRedisClient> Redis connection lost, attempting reconnection"
                        );
                        let client = redis::Client::open(config.url.clone())
                            .expect("Failed to open client during reconnection");
                        connection_manager
                            .attempt_reconnection(&client, &config)
                            .await;
                    }
                }
            }));
        } else {
            log::error!("<NewRedisClient> Failed to lock connection_monitor_task");
        }

        // Create a closure that builds the monitoring task
        let create_monitor_task = || {
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
        };

        // Create a task to return
        let task_to_return = create_monitor_task();

        // Create another task for the mutex
        if let Ok(mut guard) = self.connection_monitor_task.lock() {
            *guard = Some(create_monitor_task());
        } else {
            log::error!("<NewRedisClient> Failed to lock connection_monitor_task");
        }

        // Discard the returned task to match the expected unit type
        std::mem::drop(task_to_return);
    }

    // -------------------------------
    // Cleanup Logic in Drop
    // -------------------------------
}

impl Drop for NewRedisClient {
    fn drop(&mut self) {
        log::info!("<NewRedisClient> NewRedisClient is being dropped. Cleaning up tasks and releasing locks.");
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
        let locks = self.locks.blocking_lock();
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
            log::info!("<NewRedisClient> Released lock {}", name);
        }
    }
}

impl NewRedisClient {
    // ------------------------------------------
    // Compatibility layer for the old RedisClient API
    // ------------------------------------------

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
        // In the old client this registered the lock in memory
        let lock_key = self.service_prefix(&[name, "lock"]);
        let mut locks = self.locks.lock().await;
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

    pub fn start_periodic_tasks_compat(&mut self) {
        // This starts both presence updates and connection monitoring
        let connection_manager = self.connection_manager.clone();
        let presence_manager = self.presence_manager.clone();
        let _instance_id = self.instance_id.clone();
        let _config = self.config.clone();

        // Start presence update task
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

        let mut presence_task_guard = match self.presence_task.lock() {
            Ok(guard) => guard,
            Err(e) => {
                log::error!("<NewRedisClient> Failed to lock presence_task: {}", e);
                return;
            }
        };
        *presence_task_guard = Some(presence_task);

        // Start connection monitor task
        self.start_connection_monitor();
    }

    pub fn start_connection_monitor(&self) -> tokio::task::JoinHandle<()> {
        // Create a closure that builds the monitoring task
        let create_monitor_task = || {
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
        };

        // Create a task to return
        let task_to_return = create_monitor_task();

        // Create another task for the mutex
        if let Ok(mut guard) = self.connection_monitor_task.lock() {
            *guard = Some(create_monitor_task());
        } else {
            log::error!("<NewRedisClient> Failed to lock connection_monitor_task");
        }

        // Return the task
        task_to_return
    }

    pub fn stop_periodic_tasks(&mut self) -> anyhow::Result<()> {
        // Stop presence task
        if let Ok(mut task) = self.presence_task.lock() {
            if let Some(handle) = task.take() {
                handle.abort();
            }
        }

        // Stop connection monitor task
        if let Ok(mut task) = self.connection_monitor_task.lock() {
            if let Some(handle) = task.take() {
                handle.abort();
            }
        }

        // Stop message listener task
        if let Ok(mut task) = self.listener_task.lock() {
            if let Some(handle) = task.take() {
                handle.abort();
            }
        }

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

    // Add the key-value operations and queue operations

    pub async fn set_with_service_prefix<V: redis::ToRedisArgs + Send + Sync>(
        &self,
        key: &str,
        value: V,
    ) -> anyhow::Result<()> {
        let prefixed_key = self.service_prefix(&[key]);
        let mut conn = self.connection_manager.connection.write().await;
        conn.set::<_, _, ()>(&prefixed_key, value).await?;
        Ok(())
    }

    pub async fn get_with_service_prefix<V: redis::FromRedisValue + Send + Sync>(
        &self,
        key: &str,
    ) -> anyhow::Result<Option<V>> {
        let prefixed_key = self.service_prefix(&[key]);
        let mut conn = self.connection_manager.connection.write().await;
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
        let mut conn = self.connection_manager.connection.write().await;
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

        let mut conn = self.connection_manager.connection.write().await;
        conn.rpush::<_, _, ()>(&queue_history_key, message).await?;
        Ok(())
    }

    // ------------------------------------------
    // End of compatibility layer
    // ------------------------------------------
}
