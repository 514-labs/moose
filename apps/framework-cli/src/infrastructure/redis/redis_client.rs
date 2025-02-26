//! Redis Client Module
//!
//! This module provides a Redis client implementation with support for leader election,
//! presence updates, and message passing (Sending) and message queuing.
//!
//! Note: Make sure to set the MOOSE_REDIS_URL environment variable or the client will
//! default to "redis://127.0.0.1:6379".
use crate::utilities::retry::retry;
use anyhow::{Context, Result};
use futures::FutureExt;
use log::{debug, error, info, warn};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, FromRedisValue, RedisError, Script, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tokio_stream::StreamExt;
use uuid::Uuid; // for catch_unwind

// Internal constants that we don't expose to the user
const KEY_EXPIRATION_TTL: u64 = 3; // 3 seconds
const PRESENCE_UPDATE_INTERVAL: u64 = 1; // 1 second
const LEADERSHIP_LOCK_RENEWAL_INTERVAL: u64 = 5; // 5 seconds
const LEADERSHIP_LOCK_TTL: u64 = LEADERSHIP_LOCK_RENEWAL_INTERVAL * 3; // best practice to set lock expiration to 2-3x the renewal interval

// type alias for the callback function
use std::future::Future;
use std::pin::Pin;

type MessageCallback =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

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

#[derive(Clone)]
pub struct RedisLock {
    key: String,
    ttl: i64,
}

pub struct RedisClient {
    connection: Arc<Mutex<ConnectionManager>>,
    pub_sub: Arc<Mutex<ConnectionManager>>,
    redis_config: RedisConfig,
    service_name: String,
    instance_id: String,
    locks: HashMap<String, RedisLock>,
    presence_task: Option<JoinHandle<()>>,
    message_callbacks: Arc<Mutex<Vec<MessageCallback>>>,
    listener_task: Mutex<Option<JoinHandle<()>>>,
    connection_state: Arc<AtomicBool>,
    client: Client,
}

impl RedisClient {
    pub async fn new(service_name: String, redis_config: RedisConfig) -> Result<Self> {
        info!(
            "<RedisClient> Initializing Redis client for {} at {}",
            service_name, redis_config.url
        );

        let client = match Self::create_client(&redis_config.url) {
            Ok(client) => client,
            Err(e) => {
                error!("<RedisClient> Failed to create Redis client: {}", e);
                // Instead of returning an error, create a mock client
                warn!("<RedisClient> Falling back to mock Redis client");
                return Self::new_mock(service_name, redis_config).await;
            }
        };

        // Create connection managers with timeouts to prevent hanging
        let connection = match Self::create_connection_with_timeout(&client).await {
            Ok(conn) => Arc::new(Mutex::new(conn)),
            Err(e) => {
                error!("<RedisClient> Failed to create connection manager: {}", e);
                warn!("<RedisClient> Falling back to mock Redis client");
                return Self::new_mock(service_name, redis_config).await;
            }
        };

        let pub_sub = match Self::create_connection_with_timeout(&client).await {
            Ok(ps) => Arc::new(Mutex::new(ps)),
            Err(e) => {
                error!(
                    "<RedisClient> Failed to create pub/sub connection manager: {}",
                    e
                );
                warn!("<RedisClient> Falling back to mock Redis client");
                return Self::new_mock(service_name, redis_config).await;
            }
        };

        let instance_id = std::env::var("HOSTNAME").unwrap_or_else(|_| Uuid::new_v4().to_string());

        info!(
            "<RedisClient> Initializing Redis client for {} with instance ID: {}",
            service_name, instance_id
        );

        // Test Redis connection with timeout but don't fail if it doesn't work
        let connection_state = Arc::new(AtomicBool::new(true)); // Assume connected initially
        match tokio::time::timeout(
            Duration::from_secs(2),
            std::panic::AssertUnwindSafe(async {
                redis::cmd("PING")
                    .query_async::<_, String>(&mut *connection.lock().await)
                    .await
            })
            .catch_unwind(),
        )
        .await
        {
            Ok(Ok(Ok(response))) => {
                info!("<RedisClient> Redis connection successful: {}", response)
            }
            _ => {
                error!("<RedisClient> Redis connection failed or timed out");
                connection_state.store(false, Ordering::SeqCst);
            }
        }

        let redis_client = Self {
            connection,
            pub_sub,
            redis_config,
            instance_id,
            service_name,
            locks: HashMap::new(),
            presence_task: None,
            message_callbacks: Arc::new(Mutex::new(Vec::new())),
            listener_task: Mutex::new(None),
            connection_state,
            client,
        };

        // Start the message listener as part of initialization
        match redis_client.start_message_listener().await {
            Ok(_) => info!("<RedisClient> Successfully started message listener"),
            Err(e) => error!("<RedisClient> Failed to start message listener: {}", e),
        }

        // Start the connection monitor to handle reconnections
        let _monitor_handle = redis_client.start_connection_monitor();

        info!(
            "<RedisClient> Started {}::{}",
            redis_client.get_service_name(),
            redis_client.get_instance_id()
        );

        Ok(redis_client)
    }

    pub fn service_prefix(&self, keys: &[&str]) -> String {
        format!("{}::{}", self.redis_config.key_prefix, keys.join("::"))
    }

    pub fn instance_prefix(&self, key: &str) -> String {
        self.service_prefix(&[&self.instance_id, key])
    }

    pub async fn presence_update(&mut self) -> Result<()> {
        // Skip if Redis is not connected
        if !self.connection_state.load(Ordering::SeqCst) {
            debug!("<RedisClient> Skipping presence update as Redis is not connected");

            // Try to reconnect if Redis might be available now
            // This is an additional safety measure to ensure we reconnect as soon as possible
            let now = std::time::Instant::now();
            static mut LAST_RECONNECT_CHECK: Option<std::time::Instant> = None;

            // Only check every 5 seconds to avoid too many reconnection attempts
            let should_check = unsafe {
                match LAST_RECONNECT_CHECK {
                    Some(last) if now.duration_since(last) < Duration::from_secs(5) => false,
                    _ => {
                        LAST_RECONNECT_CHECK = Some(now);
                        true
                    }
                }
            };

            if should_check {
                debug!("<RedisClient> Checking if Redis is available now");
                // Use the actual Redis URL from the config
                let reconnect_client = match Self::create_client(&self.redis_config.url) {
                    Ok(client) => client,
                    Err(_) => {
                        return Ok(());
                    }
                };

                // Try a quick ping to see if Redis is available
                match tokio::time::timeout(Duration::from_millis(500), async {
                    match reconnect_client.get_async_connection().await {
                        Ok(mut conn) => redis::cmd("PING")
                            .query_async::<_, String>(&mut conn)
                            .await
                            .is_ok(),
                        Err(_) => false,
                    }
                })
                .await
                {
                    Ok(true) => {
                        info!("<RedisClient> Redis is now available, triggering reconnection");
                        // Trigger reconnection
                        RedisClient::attempt_reconnection(
                            &reconnect_client,
                            &self.connection,
                            &self.pub_sub,
                            &self.connection_state,
                            &self.redis_config,
                        )
                        .await;

                        // If reconnection was successful, try the presence update again
                        if self.connection_state.load(Ordering::SeqCst) {
                            info!("<RedisClient> Reconnected successfully, updating presence");
                        } else {
                            return Ok(());
                        }
                    }
                    _ => {
                        return Ok(());
                    }
                }
            } else {
                return Ok(());
            }
        }

        let key = self.instance_prefix("presence");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get current time")?
            .as_secs()
            .to_string();

        match self
            .connection
            .lock()
            .await
            .set_ex::<_, _, ()>(&key, &now, KEY_EXPIRATION_TTL)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("<RedisClient> Failed to update presence: {}", e);
                self.connection_state.store(false, Ordering::SeqCst);
                Ok(()) // Return Ok to prevent cascading failures
            }
        }
    }

    pub async fn register_lock(&mut self, name: &str, ttl: i64) -> Result<()> {
        info!("<RedisClient> Registering lock {}", name);
        let lock_key = self.service_prefix(&[name, "lock"]);
        let lock = RedisLock { key: lock_key, ttl };
        self.locks.insert(name.to_string(), lock);
        Ok(())
    }

    pub async fn attempt_lock(&self, name: &str) -> Result<bool> {
        // If Redis is not connected, we can't acquire locks
        if !self.connection_state.load(Ordering::SeqCst) {
            debug!("<RedisClient> Skipping lock attempt as Redis is not connected");
            return Ok(false);
        }

        if let Some(lock) = self.locks.get(name) {
            // This script atomically:
            // 1. Checks if the key exists
            // 2. If it doesn't exist, sets it with our instance_id and TTL
            // 3. If it does exist, checks if we own it
            // 4. Returns 1 if we acquired/own the lock, 0 otherwise
            let script = redis::Script::new(
                r#"
                local current = redis.call('GET', KEYS[1])
                if current == false then
                    -- Key doesn't exist, set it with our ID and TTL
                    redis.call('SET', KEYS[1], ARGV[1])
                    redis.call('EXPIRE', KEYS[1], ARGV[2])
                    return 1
                elseif current == ARGV[1] then
                    -- We already own the lock
                    redis.call('EXPIRE', KEYS[1], ARGV[2])
                    return 1
                else
                    -- Someone else owns the lock
                    return 0
                end
            "#,
            );

            match script
                .key(&lock.key)
                .arg(&self.instance_id)
                .arg(lock.ttl)
                .invoke_async::<_, i32>(&mut *self.connection.lock().await)
                .await
            {
                Ok(result) => Ok(result == 1),
                Err(e) => {
                    error!("<RedisClient> Failed to attempt lock: {}", e);
                    self.connection_state.store(false, Ordering::SeqCst);
                    Ok(false) // Return false to indicate we don't have the lock
                }
            }
        } else {
            Err(anyhow::anyhow!("Lock not registered"))
        }
    }

    pub async fn renew_lock(&self, name: &str) -> Result<bool> {
        if let Some(lock) = self.locks.get(name) {
            let extended: bool = self
                .connection
                .lock()
                .await
                .expire(&lock.key, lock.ttl)
                .await
                .context("Failed to renew lock")?;

            Ok(extended)
        } else {
            Err(anyhow::anyhow!("Lock not registered"))
        }
    }

    pub async fn release_lock(&mut self, name: &str) -> Result<()> {
        info!("<RedisClient> Releasing lock {}", name);
        if let Some(lock) = self.locks.get(name) {
            let script = Script::new(
                r"
                if redis.call('get', KEYS[1]) == ARGV[1] then
                    return redis.call('del', KEYS[1])
                else
                    return 0
                end
            ",
            );

            let _: () = script
                .key(&lock.key)
                .arg(&self.instance_id)
                .invoke_async(&mut *self.connection.lock().await)
                .await
                .context("Failed to release lock")?;
            self.locks.remove(name);
            Ok(())
        } else {
            info!("<RedisClient> Unable to release {} lock", name);
            Err(anyhow::anyhow!("Lock not registered"))
        }
    }

    pub async fn has_lock(&self, name: &str) -> Result<bool> {
        if let Some(lock) = self.locks.get(name) {
            let value: Option<String> = self
                .connection
                .lock()
                .await
                .get(&lock.key)
                .await
                .context("Failed to check lock")?;

            Ok(value.is_some_and(|id| id == self.instance_id))
        } else {
            Err(anyhow::anyhow!("Lock not registered"))
        }
    }

    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn get_service_name(&self) -> &str {
        &self.service_name
    }

    pub async fn send_message_to_instance(
        &self,
        message: &str,
        target_instance_id: &str,
    ) -> Result<()> {
        let channel = self.service_prefix(&[target_instance_id, "msgchannel"]);
        let _: () = self
            .pub_sub
            .lock()
            .await
            .publish(&channel, message)
            .await
            .context("Failed to send message to instance")?;
        Ok(())
    }

    pub async fn broadcast_message(&mut self, message: &str) -> Result<()> {
        let channel = self.service_prefix(&["msgchannel"]);
        let _: () = self
            .pub_sub
            .lock()
            .await
            .publish(&channel, message)
            .await
            .context("Failed to broadcast message")?;
        Ok(())
    }

    pub async fn get_queue_message(&self, feature_name: Option<&str>) -> Result<Option<String>> {
        let source_queue = match feature_name {
            Some(name) => self.service_prefix(&[name, "mqrecieved"]),
            None => self.service_prefix(&["mqrecieved"]),
        };
        let destination_queue = match feature_name {
            Some(name) => self.service_prefix(&[name, "mqprocess"]),
            None => self.service_prefix(&["mqprocess"]),
        };
        self.connection
            .lock()
            .await
            .rpoplpush(&source_queue, &destination_queue)
            .await
            .context("Failed to get queue message")
    }

    pub async fn post_queue_message(
        &self,
        message: &str,
        feature_name: Option<&str>,
    ) -> Result<()> {
        let queue = match feature_name {
            Some(name) => self.service_prefix(&[name, "mqrecieved"]),
            None => self.service_prefix(&["mqrecieved"]),
        };
        let _: () = self
            .connection
            .lock()
            .await
            .rpush(&queue, message)
            .await
            .context("Failed to post queue message")?;
        Ok(())
    }

    pub async fn mark_queue_message(
        &mut self,
        message: &str,
        success: bool,
        feature_name: Option<&str>,
    ) -> Result<()> {
        let in_progress_queue = match feature_name {
            Some(name) => self.service_prefix(&[name, "mqprocess"]),
            None => self.service_prefix(&["mqprocess"]),
        };
        let incomplete_queue = match feature_name {
            Some(name) => self.service_prefix(&[name, "mqincomplete"]),
            None => self.service_prefix(&["mqincomplete"]),
        };

        if success {
            let _: () = self
                .connection
                .lock()
                .await
                .lrem(&in_progress_queue, 0, message)
                .await
                .context("Failed to mark queue message as successful")?;
        } else {
            let mut pipeline = redis::pipe();
            pipeline
                .lrem(&in_progress_queue, 0, message)
                .rpush(&incomplete_queue, message);

            let _: () = pipeline
                .query_async(&mut *self.connection.lock().await)
                .await
                .context("Failed to mark queue message as incomplete")?;
        }
        Ok(())
    }

    pub fn start_periodic_tasks(&mut self) {
        info!("<RedisClient> Starting periodic tasks");

        self.presence_task = Some(tokio::spawn({
            let mut presence_client = self.clone();
            async move {
                let mut interval = interval(Duration::from_secs(PRESENCE_UPDATE_INTERVAL));
                loop {
                    interval.tick().await;
                    if let Err(e) = presence_client.presence_update().await {
                        error!("<RedisClient> Error updating presence: {}", e);
                    }
                }
            }
        }));

        info!("<RedisClient> Periodic tasks started");
    }

    pub fn stop_periodic_tasks(&mut self) -> Result<()> {
        if let Some(task) = self.presence_task.take() {
            task.abort();
        }
        Ok(())
    }

    pub async fn check_connection(&mut self) -> Result<()> {
        retry(
            || async {
                // We wrap the Redis ping in catch_unwind because the Redis client can sometimes panic
                // instead of returning an error when the connection is severely disrupted (e.g., network
                // partition or Redis server crash). This ensures we handle both error returns and panics
                // gracefully, preventing the panic from propagating and potentially crashing our service.
                let result = std::panic::AssertUnwindSafe(async {
                    redis::cmd("PING")
                        .query_async::<_, String>(&mut *self.connection.lock().await)
                        .await
                })
                .catch_unwind()
                .await;

                match result {
                    Ok(Ok(_)) => Ok(()),
                    Ok(Err(e)) => Err(e),
                    Err(_) => Err(redis::RedisError::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Redis connection panicked",
                    ))),
                }
            },
            |attempts, _| attempts < 3,
            Duration::from_secs(1),
        )
        .await
        .context("Failed to ping Redis server")?;
        Ok(())
    }

    pub async fn set_with_service_prefix<V: ToRedisArgs + Send + Sync>(
        &self,
        key: &str,
        value: V,
    ) -> Result<()> {
        self.connection
            .lock()
            .await
            .set::<_, V, ()>(self.service_prefix(&[key]), value)
            .await
            .context("Failed to set value in Redis")?;
        Ok(())
    }

    pub async fn get_with_service_prefix<V: FromRedisValue + Send + Sync>(
        &self,
        key: &str,
    ) -> Result<Option<V>> {
        let value: Option<V> = self
            .connection
            .lock()
            .await
            .get::<_, V>(self.service_prefix(&[key]))
            .await
            .ok();
        Ok(value)
    }

    pub async fn register_message_handler(&self, callback: Arc<dyn Fn(String) + Send + Sync>) {
        self.message_callbacks
            .lock()
            .await
            .push(Arc::new(move |message: String| {
                let callback = Arc::clone(&callback);
                Box::pin(async move {
                    callback(message);
                }) as Pin<Box<dyn Future<Output = ()> + Send>>
            }));
    }

    async fn start_message_listener(&self) -> Result<(), RedisError> {
        // If Redis is not connected, don't try to start the listener
        if !self.connection_state.load(Ordering::SeqCst) {
            warn!("<RedisClient> Not starting message listener as Redis is not connected");
            return Ok(());
        }

        let instance_channel = self.instance_prefix("msgchannel");
        let broadcast_channel = self.service_prefix(&["msgchannel"]);

        info!(
            "<RedisClient> Listening for messages on channels: {} and {}",
            instance_channel, broadcast_channel
        );

        // Create a separate PubSub connection for listening to messages
        let client = match Client::open(self.redis_config.url.clone()) {
            Ok(client) => client,
            Err(e) => {
                error!("<RedisClient> Failed to open Redis client: {}", e);
                self.connection_state.store(false, Ordering::SeqCst);
                return Err(e);
            }
        };

        // Use a timeout to prevent hanging during connection
        let pubsub_conn = match tokio::time::timeout(
            Duration::from_secs(5),
            std::panic::AssertUnwindSafe(client.get_async_connection()).catch_unwind(),
        )
        .await
        {
            Ok(Ok(Ok(conn))) => conn,
            Ok(Ok(Err(e))) => {
                error!(
                    "<RedisClient> Failed to get async connection for PubSub: {}",
                    e
                );
                self.connection_state.store(false, Ordering::SeqCst);
                return Err(e);
            }
            Ok(Err(_)) => {
                error!("<RedisClient> Panic while getting async connection for PubSub");
                self.connection_state.store(false, Ordering::SeqCst);
                return Err(RedisError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Panic while getting async connection for PubSub",
                )));
            }
            Err(_) => {
                error!("<RedisClient> Timeout while getting async connection for PubSub");
                self.connection_state.store(false, Ordering::SeqCst);
                return Err(RedisError::from(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timeout while getting async connection for PubSub",
                )));
            }
        };

        let mut pubsub = pubsub_conn.into_pubsub();

        // Use a timeout for the subscribe operation as well
        match tokio::time::timeout(
            Duration::from_secs(5),
            std::panic::AssertUnwindSafe(
                pubsub.subscribe(&[&instance_channel, &broadcast_channel]),
            )
            .catch_unwind(),
        )
        .await
        {
            Ok(Ok(Ok(_))) => {}
            Ok(Ok(Err(e))) => {
                error!("<RedisClient> Failed to subscribe to channels: {}", e);
                self.connection_state.store(false, Ordering::SeqCst);
                return Err(e);
            }
            Ok(Err(_)) => {
                error!("<RedisClient> Panic while subscribing to channels");
                self.connection_state.store(false, Ordering::SeqCst);
                return Err(RedisError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Panic while subscribing to channels",
                )));
            }
            Err(_) => {
                error!("<RedisClient> Timeout while subscribing to channels");
                self.connection_state.store(false, Ordering::SeqCst);
                return Err(RedisError::from(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timeout while subscribing to channels",
                )));
            }
        }

        let callback = self.message_callbacks.clone();
        let connection_state = self.connection_state.clone();
        let redis_config = self.redis_config.clone();
        let instance_channel_clone = instance_channel.clone();
        let broadcast_channel_clone = broadcast_channel.clone();

        let listener_task = tokio::spawn(async move {
            loop {
                // If Redis is disconnected, sleep and try again
                if !connection_state.load(Ordering::SeqCst) {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                }

                // Wrap the entire message processing in catch_unwind to prevent panics
                let result = std::panic::AssertUnwindSafe(async {
                    // Use a timeout for getting the message stream
                    let pubsub_stream_result = tokio::time::timeout(
                        Duration::from_secs(2),
                        std::panic::AssertUnwindSafe(async { pubsub.on_message() }).catch_unwind(),
                    )
                    .await;

                    let mut pubsub_stream = match pubsub_stream_result {
                        Ok(Ok(stream)) => stream,
                        _ => {
                            return Err::<(), _>(redis::RedisError::from(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Failed to get pubsub stream",
                            )));
                        }
                    };

                    while let Some(msg) = pubsub_stream.next().await {
                        if let Ok(payload) = msg.get_payload::<String>() {
                            info!("<RedisClient> Received pubsub message: {}", payload);
                            let callbacks = callback.lock().await.clone();
                            for cb in callbacks {
                                cb(payload.clone()).await;
                            }
                        } else {
                            warn!("<RedisClient> Failed to get payload from message");
                        }
                    }

                    // If we exit the loop, the connection might be broken
                    Err::<(), _>(redis::RedisError::from(std::io::Error::new(
                        std::io::ErrorKind::ConnectionAborted,
                        "PubSub stream ended unexpectedly",
                    )))
                })
                .catch_unwind()
                .await;

                match result {
                    Ok(Err(_)) | Err(_) => {
                        // Connection failed or panicked
                        error!("<RedisClient> PubSub connection lost or panicked, will retry");
                        connection_state.store(false, Ordering::SeqCst);

                        // Wait before trying to reconnect
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                        // Try to reestablish the connection with improved error handling
                        if let Ok(client) = Client::open(redis_config.url.clone()) {
                            // Use a timeout for reconnection
                            let reconnect_result = tokio::time::timeout(
                                Duration::from_secs(5),
                                std::panic::AssertUnwindSafe(client.get_async_connection())
                                    .catch_unwind(),
                            )
                            .await;

                            match reconnect_result {
                                Ok(Ok(Ok(conn))) => {
                                    pubsub = conn.into_pubsub();

                                    // Use a timeout for resubscribing
                                    let resubscribe_result = tokio::time::timeout(
                                        Duration::from_secs(5),
                                        std::panic::AssertUnwindSafe(pubsub.subscribe(&[
                                            &instance_channel_clone,
                                            &broadcast_channel_clone,
                                        ]))
                                        .catch_unwind(),
                                    )
                                    .await;

                                    match resubscribe_result {
                                        Ok(Ok(Ok(_))) => {
                                            info!("<RedisClient> Successfully reconnected PubSub");
                                            connection_state.store(true, Ordering::SeqCst);
                                        }
                                        _ => {
                                            error!(
                                                "<RedisClient> Failed to resubscribe to channels"
                                            );
                                        }
                                    }
                                }
                                _ => {
                                    error!("<RedisClient> Failed to reestablish PubSub connection");
                                }
                            }
                        }
                    }
                    _ => {
                        // This shouldn't happen, but just in case
                        warn!("<RedisClient> PubSub stream ended unexpectedly");
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });

        // Store the listener task
        *self.listener_task.lock().await = Some(listener_task);

        Ok(())
    }

    pub async fn check_and_renew_lock(&self, name: &str) -> Result<(bool, bool)> {
        if let Some(lock) = self.locks.get(name) {
            let script = redis::Script::new(
                r#"
                local current = redis.call('GET', KEYS[1])
                if current == false then
                    -- Key doesn't exist, acquire it
                    redis.call('SET', KEYS[1], ARGV[1])
                    redis.call('EXPIRE', KEYS[1], ARGV[2])
                    return {1, 1}  -- has_lock=true, is_new=true
                elseif current == ARGV[1] then
                    -- We own it, renew it
                    redis.call('EXPIRE', KEYS[1], ARGV[2])
                    return {1, 0}  -- has_lock=true, is_new=false
                else
                    -- Someone else owns it
                    return {0, 0}  -- has_lock=false, is_new=false
                end
            "#,
            );

            let result: Vec<i32> = script
                .key(&lock.key)
                .arg(&self.instance_id)
                .arg(lock.ttl)
                .invoke_async(&mut *self.connection.lock().await)
                .await
                .context("Failed to check and renew lock")?;

            Ok((result[0] == 1, result[1] == 1))
        } else {
            Err(anyhow::anyhow!("Lock not registered"))
        }
    }

    pub async fn restart_redis_tasks(&mut self) -> Result<(), RedisError> {
        info!("<RedisClient> Restarting Redis-dependent tasks after reconnection");

        // Stop any existing tasks
        if let Err(e) = self.stop_periodic_tasks() {
            warn!("<RedisClient> Error stopping periodic tasks: {}", e);
        }

        // Restart presence updates
        self.start_periodic_tasks();

        // Restart message listener
        if let Err(e) = self.start_message_listener().await {
            warn!("<RedisClient> Error restarting message listener: {}", e);
            return Err(e);
        }

        // Re-register leadership lock if it was previously registered
        if self.locks.contains_key("leadership") {
            if let Err(e) = self
                .register_lock("leadership", LEADERSHIP_LOCK_TTL as i64)
                .await
            {
                warn!("<RedisClient> Error re-registering leadership lock: {}", e);
            } else {
                info!("<RedisClient> Successfully re-registered leadership lock");
            }
        }

        info!("<RedisClient> Successfully restarted Redis-dependent tasks");
        Ok(())
    }

    pub fn start_connection_monitor(&self) -> tokio::task::JoinHandle<()> {
        let client = self.client.clone();
        let connection = self.connection.clone();
        let pub_sub = self.pub_sub.clone();
        let connection_state = self.connection_state.clone();
        let redis_config = self.redis_config.clone();
        let service_name = self.service_name.clone();
        let instance_id = self.instance_id.clone();

        tokio::spawn(async move {
            // Start with a 5 second interval, but use exponential backoff when disconnected
            let base_interval = 5;
            let mut consecutive_failures: u32 = 0;
            let mut was_disconnected = !connection_state.load(Ordering::SeqCst);

            loop {
                // Calculate backoff interval (5s, 10s, 20s, 40s, max 60s)
                let backoff_secs = if consecutive_failures > 0 {
                    std::cmp::min(base_interval * 2u64.pow(consecutive_failures), 60)
                } else {
                    base_interval
                };

                // Sleep for the calculated interval
                tokio::time::sleep(Duration::from_secs(backoff_secs)).await;

                // Track the current connection state
                let is_connected = connection_state.load(Ordering::SeqCst);

                // Only log connection checks if we're disconnected to avoid spamming logs
                if !is_connected {
                    info!(
                        "<RedisClient> Checking Redis connection for {}::{} (attempt: {}, backoff: {}s)",
                        service_name, instance_id, consecutive_failures + 1, backoff_secs
                    );
                }

                // Always attempt to reconnect, regardless of current connection state
                // This ensures that mock clients will also try to connect to a real Redis server
                match tokio::time::timeout(
                    Duration::from_secs(3), // Longer timeout for reconnection attempts
                    Self::attempt_reconnection(
                        &client,
                        &connection,
                        &pub_sub,
                        &connection_state,
                        &redis_config,
                    ),
                )
                .await
                {
                    Ok(_) => {
                        // Check if we've transitioned from disconnected to connected
                        let is_connected_now = connection_state.load(Ordering::SeqCst);

                        if is_connected_now {
                            // Reset failure counter on successful connection
                            consecutive_failures = 0;

                            if was_disconnected {
                                info!(
                                    "<RedisClient> Successfully reconnected to Redis for {}::{}",
                                    service_name, instance_id
                                );
                            }
                        } else {
                            // Increment failure counter on failed connection attempt
                            consecutive_failures = consecutive_failures.saturating_add(1);
                        }

                        was_disconnected = !is_connected_now;
                    }
                    Err(e) => {
                        // Timeout occurred
                        consecutive_failures = consecutive_failures.saturating_add(1);

                        if is_connected {
                            warn!(
                                "<RedisClient> Reconnection attempt timed out for {}::{}: {}",
                                service_name, instance_id, e
                            );
                            // Mark as disconnected since we couldn't verify the connection
                            connection_state.store(false, Ordering::SeqCst);
                            was_disconnected = true;
                        }
                    }
                }
            }
        })
    }

    async fn attempt_reconnection(
        client: &Client,
        connection: &Arc<Mutex<ConnectionManager>>,
        pub_sub: &Arc<Mutex<ConnectionManager>>,
        connection_state: &Arc<AtomicBool>,
        redis_config: &RedisConfig,
    ) {
        // First, check if we're already connected - if so, just verify the connection
        if connection_state.load(Ordering::SeqCst) {
            // Verify the connection is still good with a ping
            let ping_result = std::panic::AssertUnwindSafe(async {
                matches!(
                    tokio::time::timeout(Duration::from_millis(500), async {
                        // Use try_lock instead of lock to avoid deadlocks
                        match connection.try_lock() {
                            Ok(mut conn) => {
                                redis::cmd("PING")
                                    .query_async::<_, String>(&mut *conn)
                                    .await
                            }
                            Err(_) => {
                                // If we can't get the lock, assume the connection is still good
                                Ok("LOCKED".to_string())
                            }
                        }
                    })
                    .await,
                    Ok(Ok(_))
                )
            })
            .catch_unwind()
            .await;

            match ping_result {
                Ok(true) => {
                    // Connection is still good, nothing to do
                    return;
                }
                _ => {
                    // Connection is not good, mark as disconnected and continue with reconnection
                    connection_state.store(false, Ordering::SeqCst);
                    debug!("<RedisClient> Connection verification failed, attempting reconnection");
                }
            }
        }

        debug!(
            "<RedisClient> Attempting to reconnect to Redis at {}",
            redis_config.url
        );

        // Try to establish a new connection with a more cautious approach
        // First, check if Redis is actually available with a simple ping
        let ping_result = std::panic::AssertUnwindSafe(async {
            (tokio::time::timeout(Duration::from_secs(2), async {
                match client.get_async_connection().await {
                    Ok(mut conn) => {
                        match redis::cmd("PING").query_async::<_, String>(&mut conn).await {
                            Ok(_) => {
                                debug!("<RedisClient> Redis server is available");
                                // Explicitly close the connection to avoid leaking file descriptors
                                drop(conn);
                                true
                            }
                            Err(e) => {
                                debug!("<RedisClient> Redis ping failed: {}", e);
                                // Explicitly close the connection to avoid leaking file descriptors
                                drop(conn);
                                false
                            }
                        }
                    }
                    Err(e) => {
                        debug!("<RedisClient> Failed to get connection for ping: {}", e);
                        false
                    }
                }
            })
            .await)
                .unwrap_or_default()
        })
        .catch_unwind()
        .await;

        // If Redis is not available, don't waste time trying to create connection managers
        let redis_available = match ping_result {
            Ok(true) => true,
            _ => {
                debug!(
                    "<RedisClient> Redis server is not available, skipping reconnection attempt"
                );
                connection_state.store(false, Ordering::SeqCst);
                return;
            }
        };

        if !redis_available {
            return;
        }

        // Redis is available, so create new connections
        info!("<RedisClient> Redis server is available, creating new connections");

        // Create new connections with a longer timeout
        let new_connection_result = tokio::time::timeout(
            Duration::from_secs(5),
            Self::create_connection_safely(client),
        )
        .await
        .ok()
        .and_then(|r| r.ok());

        let new_pubsub_result = tokio::time::timeout(
            Duration::from_secs(5),
            Self::create_connection_safely(client),
        )
        .await
        .ok()
        .and_then(|r| r.ok());

        // Use a match statement to avoid moving values
        match (new_connection_result, new_pubsub_result) {
            (Some(new_conn), Some(new_pubsub)) => {
                // Replace the connections - IMPORTANT: properly close old connections first
                let conn_replaced = match connection.try_lock() {
                    Ok(mut conn_guard) => {
                        // Explicitly drop the old connection before replacing it
                        let old_conn = std::mem::replace(&mut *conn_guard, new_conn);
                        drop(old_conn);
                        true
                    }
                    Err(_) => {
                        warn!(
                            "<RedisClient> Could not acquire lock on connection, will retry later"
                        );
                        // Drop the new connection to avoid leaking resources
                        drop(new_conn);
                        false
                    }
                };

                let pubsub_replaced = match pub_sub.try_lock() {
                    Ok(mut pubsub_guard) => {
                        // Explicitly drop the old connection before replacing it
                        let old_pubsub = std::mem::replace(&mut *pubsub_guard, new_pubsub);
                        drop(old_pubsub);
                        true
                    }
                    Err(_) => {
                        warn!("<RedisClient> Could not acquire lock on pub_sub, will retry later");
                        // Drop the new connection to avoid leaking resources
                        drop(new_pubsub);
                        false
                    }
                };

                // Only proceed if we successfully replaced both connections
                if conn_replaced && pubsub_replaced {
                    info!("<RedisClient> Successfully replaced Redis connections");
                    // IMPORTANT: Update the connection state AFTER successfully replacing connections
                    connection_state.store(true, Ordering::SeqCst);
                } else {
                    error!("<RedisClient> Could not replace all connections");
                    connection_state.store(false, Ordering::SeqCst);
                }
            }
            (Some(conn), None) => {
                error!("<RedisClient> Failed to create new pub/sub connection");
                connection_state.store(false, Ordering::SeqCst);
                // Drop the connection to avoid leaking resources
                drop(conn);
            }
            (None, Some(pubsub)) => {
                error!("<RedisClient> Failed to create new connection");
                connection_state.store(false, Ordering::SeqCst);
                // Drop the pub/sub connection to avoid leaking resources
                drop(pubsub);
            }
            (None, None) => {
                error!("<RedisClient> Failed to create new connections");
                connection_state.store(false, Ordering::SeqCst);
            }
        }
    }

    // Helper method to create a connection with timeout
    async fn create_connection_with_timeout(
        client: &Client,
    ) -> Result<ConnectionManager, RedisError> {
        // First try to get a simple connection to verify Redis is available
        // This helps prevent resource leaks by failing fast if Redis is unavailable
        match tokio::time::timeout(Duration::from_secs(2), client.get_async_connection()).await {
            Ok(Ok(conn)) => {
                // Successfully connected, explicitly drop the connection
                drop(conn);
            }
            Ok(Err(e)) => {
                error!("<RedisClient> Failed to establish test connection: {}", e);
                return Err(e);
            }
            Err(_) => {
                error!("<RedisClient> Timeout while establishing test connection");
                return Err(RedisError::from(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timeout while establishing test connection",
                )));
            }
        }

        // Now create the actual connection manager
        match tokio::time::timeout(
            Duration::from_secs(5),
            std::panic::AssertUnwindSafe(ConnectionManager::new(client.clone())).catch_unwind(),
        )
        .await
        {
            Ok(Ok(Ok(conn))) => Ok(conn),
            Ok(Ok(Err(e))) => {
                error!("<RedisClient> Error creating connection: {}", e);
                Err(e)
            }
            Ok(Err(_)) => {
                error!("<RedisClient> Panic while creating connection");
                Err(RedisError::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Panic while creating connection",
                )))
            }
            Err(_) => {
                error!("<RedisClient> Timeout while creating connection");
                Err(RedisError::from(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Timeout while creating connection",
                )))
            }
        }
    }

    // Helper method to create a connection with timeout
    async fn create_connection_safely(client: &Client) -> Result<ConnectionManager, RedisError> {
        Self::create_connection_with_timeout(client).await
    }

    // Helper method to test a connection with a PING
    #[allow(dead_code)]
    async fn test_connection(connection: &Arc<Mutex<ConnectionManager>>) -> bool {
        let ping_result = std::panic::AssertUnwindSafe(async {
            matches!(
                tokio::time::timeout(Duration::from_secs(2), async {
                    let mut conn = connection.lock().await;
                    redis::cmd("PING")
                        .query_async::<_, String>(&mut *conn)
                        .await
                })
                .await,
                Ok(Ok(_))
            )
        })
        .catch_unwind()
        .await;

        ping_result.unwrap_or_default()
    }

    pub fn is_connected(&self) -> bool {
        self.connection_state.load(Ordering::SeqCst)
    }

    pub fn set_connection_state(&self, connected: bool) {
        self.connection_state.store(connected, Ordering::SeqCst);
        if connected {
            info!("<RedisClient> Connection state manually set to connected");
        } else {
            warn!("<RedisClient> Connection state manually set to disconnected");
        }
    }

    // Create a mock Redis client that doesn't require an actual Redis connection
    pub async fn new_mock(service_name: String, redis_config: RedisConfig) -> Result<Self> {
        info!(
            "<RedisClient> Creating mock Redis client for {}",
            service_name
        );

        let instance_id = std::env::var("HOSTNAME").unwrap_or_else(|_| Uuid::new_v4().to_string());

        // Create a dummy client just to satisfy the type system
        let client = match Client::open("redis://localhost:6379") {
            Ok(client) => client,
            Err(e) => {
                warn!("<RedisClient> Failed to create dummy client: {}", e);
                // Create a client with a different URL as a fallback
                Client::open("redis://127.0.0.1:6379").unwrap_or_else(|_| {
                    // This should never fail, but just in case
                    Client::open("redis://localhost:6379/").expect("Failed to create dummy client")
                })
            }
        };

        // Create dummy connection managers that will never be used for real operations
        // We'll create empty connection managers and set connection_state to false
        let connection = Arc::new(Mutex::new(
            match ConnectionManager::new(client.clone()).await {
                Ok(conn) => conn,
                Err(e) => {
                    // If we can't create a connection manager, log the error and create a safer fallback
                    warn!(
                        "<RedisClient> Failed to create mock connection manager: {}",
                        e
                    );
                    // Create a completely new client as a fallback
                    let fallback_client =
                        Client::open("redis://localhost:6379").unwrap_or_else(|_| {
                            Client::open("redis://127.0.0.1:6379").unwrap_or_else(|_| {
                                // Last resort - create an in-memory client that will never connect
                                warn!("<RedisClient> Creating last-resort in-memory client");
                                Client::open("redis://127.0.0.1:1")
                                    .expect("Failed to create in-memory client")
                            })
                        });

                    // Try to create a connection manager with the fallback client
                    match ConnectionManager::new(fallback_client).now_or_never() {
                        Some(Ok(conn)) => conn,
                        _ => {
                            warn!("<RedisClient> Using emergency fallback for connection manager");
                            // Create a minimal ConnectionManager that won't panic but will fail operations
                            ConnectionManager::new(
                                Client::open("redis://127.0.0.1:1")
                                    .expect("Failed to create in-memory client"),
                            )
                            .now_or_never()
                            .unwrap()
                            .expect("Failed to create fallback connection manager")
                        }
                    }
                }
            },
        ));

        // Create a separate pub_sub connection manager to avoid lock contention
        let pub_sub = Arc::new(Mutex::new(
            match ConnectionManager::new(client.clone()).await {
                Ok(conn) => conn,
                Err(e) => {
                    // If we can't create a connection manager, log the error and create a safer fallback
                    warn!(
                        "<RedisClient> Failed to create mock pub/sub connection manager: {}",
                        e
                    );
                    // Create a completely new client as a fallback
                    let fallback_client = Client::open("redis://localhost:6379").unwrap_or_else(|_| {
                        Client::open("redis://127.0.0.1:6379").unwrap_or_else(|_| {
                            // Last resort - create an in-memory client that will never connect
                            warn!("<RedisClient> Creating last-resort in-memory client for pub/sub");
                            Client::open("redis://127.0.0.1:1").expect("Failed to create in-memory client")
                        })
                    });

                    // Try to create a connection manager with the fallback client
                    match ConnectionManager::new(fallback_client).now_or_never() {
                        Some(Ok(conn)) => conn,
                        _ => {
                            warn!("<RedisClient> Using emergency fallback for pub/sub connection manager");
                            // Create a minimal ConnectionManager that won't panic but will fail operations
                            ConnectionManager::new(
                                Client::open("redis://127.0.0.1:1")
                                    .expect("Failed to create in-memory client"),
                            )
                            .now_or_never()
                            .unwrap()
                            .expect("Failed to create fallback pub/sub connection manager")
                        }
                    }
                }
            },
        ));

        // IMPORTANT: Create a client with the ACTUAL Redis URL from the config
        // This ensures that when Redis becomes available, we can connect to it
        let real_client = match Self::create_client(&redis_config.url) {
            Ok(client) => client,
            Err(_) => {
                // If we can't create a client with the real URL, fall back to the dummy client
                warn!("<RedisClient> Failed to create client with real URL, using dummy client");
                client.clone()
            }
        };

        let connection_state = Arc::new(AtomicBool::new(false)); // Mark as disconnected

        let redis_client = Self {
            connection,
            pub_sub,
            redis_config,
            instance_id,
            service_name,
            locks: HashMap::new(),
            presence_task: None,
            message_callbacks: Arc::new(Mutex::new(Vec::new())),
            listener_task: Mutex::new(None),
            connection_state,
            client: real_client, // Use the real client for reconnection attempts
        };

        info!(
            "<RedisClient> Created mock client {}::{}",
            redis_client.get_service_name(),
            redis_client.get_instance_id()
        );

        // Start the connection monitor to periodically check if Redis becomes available
        let _monitor_handle = redis_client.start_connection_monitor();

        Ok(redis_client)
    }

    // Helper method to create a client with proper error handling
    fn create_client(url: &str) -> Result<Client, RedisError> {
        match Client::open(url) {
            Ok(client) => Ok(client),
            Err(e) => {
                error!("<RedisClient> Failed to create Redis client: {}", e);
                Err(e)
            }
        }
    }
}

impl Clone for RedisClient {
    // FIXME: some state (the stateful connection) is shared,
    // some state (the locks map) is not,
    // some state (the listener task) is gone
    // this is not a good abstraction for cloning
    fn clone(&self) -> Self {
        Self {
            connection: Arc::clone(&self.connection),
            pub_sub: Arc::clone(&self.pub_sub),
            redis_config: self.redis_config.clone(),
            instance_id: self.instance_id.clone(),
            service_name: self.service_name.clone(),
            locks: self.locks.clone(),
            presence_task: None,
            message_callbacks: Arc::clone(&self.message_callbacks),
            listener_task: Mutex::new(None),
            connection_state: Arc::clone(&self.connection_state),
            client: self.client.clone(),
        }
    }
}

impl Drop for RedisClient {
    fn drop(&mut self) {
        info!("<RedisClient> is being dropped");
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            if let Err(e) = self.stop_periodic_tasks() {
                error!("<RedisClient> Error stopping periodic tasks: {}", e);
            }
            if !self.locks.is_empty() {
                let mut self_clone = self.clone();
                rt.spawn(async move {
                    let lock_names: Vec<_> = self_clone.locks.keys().cloned().collect();
                    for name in lock_names {
                        if let Err(e) = self_clone.release_lock(&name).await {
                            error!("<RedisClient> Error releasing lock {}: {}", name, e);
                        }
                    }
                });
            }
        } else {
            error!("<RedisClient> Failed to get current runtime handle in RedisClient::drop");
        }
        if let Ok(mut guard) = self.listener_task.try_lock() {
            if let Some(task) = guard.take() {
                task.abort();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio;

    #[ignore]
    #[tokio::test]
    async fn test_redis_operations() {
        let client = RedisClient::new(
            "test_service".to_string(),
            RedisConfig {
                url: "redis://127.0.0.1:6379".to_string(),
                key_prefix: "MS".to_string(),
            },
        )
        .await
        .expect("Failed to create Redis client");

        // Test set and get
        let _: () = client
            .connection
            .lock()
            .await
            .set("test_key", "test_value")
            .await
            .expect("Failed to set value");
        let result: Option<String> = client
            .connection
            .lock()
            .await
            .get("test_key")
            .await
            .expect("Failed to get value");
        assert_eq!(result, Some("test_value".to_string()));

        // Test delete
        let _: () = client
            .connection
            .lock()
            .await
            .del("test_key")
            .await
            .expect("Failed to delete key");
        let result: Option<String> = client
            .connection
            .lock()
            .await
            .get("test_key")
            .await
            .expect("Failed to get value after deletion");
        assert_eq!(result, None);

        // Test set_ex
        let _: () = client
            .connection
            .lock()
            .await
            .set_ex("test_ex_key", "test_ex_value", 1)
            .await
            .expect("Failed to set value with expiry");
        let result: Option<String> = client
            .connection
            .lock()
            .await
            .get("test_ex_key")
            .await
            .expect("Failed to get value with expiry");
        assert_eq!(result, Some("test_ex_value".to_string()));
        tokio::time::sleep(Duration::from_secs(2)).await;
        let result: Option<String> = client
            .connection
            .lock()
            .await
            .get("test_ex_key")
            .await
            .expect("Failed to get expired value");
        assert_eq!(result, None);

        // Test rpush and rpoplpush
        let _: () = client
            .connection
            .lock()
            .await
            .rpush("source_list", "item1")
            .await
            .expect("Failed to push to list");
        let _: () = client
            .connection
            .lock()
            .await
            .rpush("source_list", "item2")
            .await
            .expect("Failed to push to list");
        let result: Option<String> = client
            .connection
            .lock()
            .await
            .rpoplpush("source_list", "dest_list")
            .await
            .expect("Failed to rpoplpush");
        assert_eq!(result, Some("item2".to_string()));

        // Test lrem
        let _: () = client
            .connection
            .lock()
            .await
            .lrem("dest_list", 0, "item2")
            .await
            .expect("Failed to remove from list");
        let result: Option<String> = client
            .connection
            .lock()
            .await
            .rpoplpush("dest_list", "source_list")
            .await
            .expect("Failed to rpoplpush after lrem");
        assert_eq!(result, None);

        // Cleanup
        let _: () = client
            .connection
            .lock()
            .await
            .del("dest_list")
            .await
            .expect("Failed to delete dest_list");
        let _: () = client
            .connection
            .lock()
            .await
            .del("source_list")
            .await
            .expect("Failed to delete source_list");
    }
}
