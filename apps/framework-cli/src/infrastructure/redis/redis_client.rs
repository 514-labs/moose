//! Redis Client Module
//!
//! This module provides a Redis client implementation with support for leader election,
//! presence updates, and message passing (Sending) and message queuing.
//!
//! Note: Make sure to set the MOOSE_REDIS_URL environment variable or the client will
//! default to "redis://127.0.0.1:6379".
use anyhow::{Context, Result};
use futures::StreamExt;
use log::{error, info};
use redis::aio::{ConnectionManager, PubSub};
use redis::{AsyncCommands, Client, Script, ToRedisArgs};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use uuid::Uuid;

// Internal constants that we don't expose to the user
const KEY_EXPIRATION_TTL: u64 = 3; // 3 seconds
const PRESENCE_UPDATE_INTERVAL: u64 = 1; // 1 second

// type alias for the callback function
use std::future::Future;
use std::pin::Pin;

type MessageCallback =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// The connection status of the Redis client.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

/// Configuration required to connect to Redis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RedisConfig {
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
    pub_sub_connection: Arc<Mutex<PubSub>>,
    redis_config: RedisConfig,
    service_name: String,
    instance_id: String,
    locks: HashMap<String, RedisLock>,
    presence_task: Option<JoinHandle<()>>,
    message_callbacks: Arc<Mutex<Vec<MessageCallback>>>,
    listener_task: Mutex<Option<JoinHandle<()>>>,
    connection_status: Arc<Mutex<ConnectionStatus>>,
}

impl RedisClient {
    pub async fn new(service_name: String, redis_config: RedisConfig) -> Result<Self> {
        let client =
            Client::open(redis_config.url.clone()).context("Failed to create Redis client")?;

        // Create a connection manager for normal commands.
        let connection_manager = ConnectionManager::new(client.clone())
            .await
            .context("Failed to create Redis connection manager")?;

        // Create a separate pub/sub connection using get_async_pubsub
        let pub_sub_connection = client
            .get_async_pubsub()
            .await
            .context("Failed to create Redis pub/sub connection")?;

        let instance_id = std::env::var("HOSTNAME").unwrap_or_else(|_| Uuid::new_v4().to_string());

        info!(
            "<RedisClient> Initializing Redis client for {} with instance ID: {}",
            service_name, instance_id
        );

        let mut connection_test = connection_manager.clone();
        match redis::cmd("PING")
            .query_async::<String>(&mut connection_test)
            .await
        {
            Ok(response) => info!("<RedisClient> Redis connection successful: {}", response),
            Err(e) => error!("<RedisClient> Redis connection failed: {}", e),
        }

        let mut client_instance = Self {
            connection: Arc::new(Mutex::new(connection_manager)),
            pub_sub_connection: Arc::new(Mutex::new(pub_sub_connection)),
            redis_config,
            service_name,
            instance_id,
            locks: HashMap::new(),
            presence_task: None,
            message_callbacks: Arc::new(Mutex::new(Vec::new())),
            listener_task: Mutex::new(None),
            connection_status: Arc::new(Mutex::new(ConnectionStatus::Connected)),
        };

        // Start periodic tasks (e.g., presence updates and reconnection monitoring)
        client_instance.start_periodic_tasks().await;

        // Optionally, start a message listener here.
        match client_instance.start_message_listener().await {
            Ok(_) => info!("<RedisClient> Successfully started message listener"),
            Err(e) => error!("<RedisClient> Failed to start message listener: {}", e),
        }

        info!(
            "<RedisClient> Started {}::{}",
            client_instance.get_service_name(),
            client_instance.get_instance_id()
        );

        Ok(client_instance)
    }

    /// Starts background tasks that periodically update the client's "presence" and attempt
    /// to reconnect if the connection is lost.
    pub async fn start_periodic_tasks(&mut self) {
        info!("Starting periodic tasks");

        let client_clone = self.clone();
        self.presence_task = Some(tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(PRESENCE_UPDATE_INTERVAL));
            loop {
                interval.tick().await;
                if let Err(e) = client_clone.presence_update().await {
                    error!("Error updating presence: {}", e);
                    // Mark connection as disconnected.
                    {
                        let mut status = client_clone.connection_status.lock().await;
                        *status = ConnectionStatus::Disconnected;
                    }
                    // Attempt to reconnect.
                    if let Err(e) = client_clone.check_connection().await {
                        error!("Failed to reconnect to Redis: {}", e);
                    }
                }
            }
        }));

        info!("Periodic tasks started");
    }

    pub async fn presence_update(&self) -> Result<(), anyhow::Error> {
        if let Err(e) = self.check_connection_health().await {
            error!("Connection health check failed: {}", e);
            return Err(e);
        }

        let key = format!("presence:{}", self.instance_id);
        let _: () = self
            .connection
            .lock()
            .await
            .set_ex(&key, "online", KEY_EXPIRATION_TTL)
            .await?;

        let mut status = self.connection_status.lock().await;
        *status = ConnectionStatus::Connected;
        Ok(())
    }

    /// Checks the connection by issuing a PING command.
    /// If the PING succeeds, the connection status is updated to Connected.
    pub async fn check_connection(&self) -> Result<()> {
        info!("<RedisClient> Checking connection status...");
        let mut conn = self.connection.lock().await;

        match redis::cmd("PING").query_async::<String>(&mut *conn).await {
            Ok(response) => {
                info!("<RedisClient> Connection healthy: {}", response);
                let mut status = self.connection_status.lock().await;
                *status = ConnectionStatus::Connected;
                Ok(())
            }
            Err(e) => {
                error!("<RedisClient> Connection unhealthy: {}", e);
                let mut status = self.connection_status.lock().await;
                *status = ConnectionStatus::Disconnected;
                Err(e.into())
            }
        }
    }

    async fn resubscribe_channels(&self, channels: &[String]) -> Result<()> {
        let mut retries = 0;
        let mut delay = Duration::from_secs(1);
        while retries < 3 {
            match self
                .pub_sub_connection
                .lock()
                .await
                .subscribe(channels)
                .await
            {
                Ok(_) => return Ok(()),
                Err(e) => {
                    error!(
                        "<RedisClient> Failed to resubscribe: {}, attempt {}/3",
                        e,
                        retries + 1
                    );
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                    retries += 1;
                }
            }
        }
        Err(anyhow::anyhow!("Failed to resubscribe after 3 attempts"))
    }

    pub async fn start_message_listener(&self) -> Result<()> {
        let instance_channel = self.instance_prefix("msgchannel");
        let broadcast_channel = self.service_prefix(&["msgchannel"]);
        let channels = vec![instance_channel.clone(), broadcast_channel.clone()];

        loop {
            info!("<RedisClient> Starting pub/sub listener...");

            // Check connection health first
            if let Err(e) = self.check_connection_health().await {
                error!("<RedisClient> Connection check failed: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            info!("<RedisClient> Connection healthy");

            // Then attempt to resubscribe
            if let Err(e) = self.resubscribe_channels(&channels).await {
                error!("<RedisClient> Failed to resubscribe channels: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
            info!("<RedisClient> Successfully subscribed to channels");

            let mut pubsub = self.pub_sub_connection.lock().await;
            let mut stream = pubsub.on_message();
            info!("<RedisClient> Message stream established");

            while let Some(msg) = stream.next().await {
                match msg.get_payload::<String>() {
                    Ok(payload) => {
                        info!("<RedisClient> Received message: {}", payload);
                    }
                    Err(e) => error!("<RedisClient> Failed to parse message: {}", e),
                }
            }

            error!("<RedisClient> Message stream ended, attempting to reconnect...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    pub fn get_service_name(&self) -> &str {
        &self.service_name
    }

    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn service_prefix(&self, keys: &[&str]) -> String {
        format!("{}::{}", self.redis_config.key_prefix, keys.join("::"))
    }

    pub fn instance_prefix(&self, key: &str) -> String {
        self.service_prefix(&[&self.instance_id, key])
    }

    pub async fn register_lock(&mut self, name: &str, ttl: i64) -> Result<()> {
        info!("<RedisClient> Registering lock {}", name);
        let lock_key = self.service_prefix(&[name, "lock"]);
        let lock = RedisLock { key: lock_key, ttl };
        self.locks.insert(name.to_string(), lock);
        Ok(())
    }

    pub async fn attempt_lock(&self, name: &str) -> Result<bool> {
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

            let result: i32 = script
                .key(&lock.key)
                .arg(&self.instance_id)
                .arg(lock.ttl)
                .invoke_async(&mut *self.connection.lock().await)
                .await
                .context("Failed to attempt lock")?;

            Ok(result == 1)
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

    pub async fn send_message_to_instance(
        &self,
        message: &str,
        target_instance_id: &str,
    ) -> Result<()> {
        let channel = self.service_prefix(&[target_instance_id, "msgchannel"]);
        let _: () = self
            .connection
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
            .connection
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

    pub fn stop_periodic_tasks(&mut self) -> Result<()> {
        if let Some(task) = self.presence_task.take() {
            task.abort();
        }
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

    pub async fn check_and_renew_lock(&self, name: &str) -> Result<(bool, bool)> {
        if let Some(lock) = self.locks.get(name) {
            let script = Script::new(
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

    async fn check_connection_health(&self) -> Result<()> {
        let mut conn = self.connection.lock().await;
        match redis::cmd("PING").query_async::<String>(&mut *conn).await {
            Ok(_) => {
                info!("<RedisClient> Connection healthy");
                Ok(())
            }
            Err(e) => {
                error!("<RedisClient> Connection unhealthy: {}", e);
                self.attempt_reconnect().await
            }
        }
    }

    async fn attempt_reconnect(&self) -> Result<()> {
        let client = Client::open(self.redis_config.url.clone())?;

        // Attempt to recreate connection manager
        let connection_manager = ConnectionManager::new(client.clone()).await?;
        *self.connection.lock().await = connection_manager;

        // Attempt to recreate pubsub connection
        let pub_sub_connection = client.get_async_pubsub().await?;
        *self.pub_sub_connection.lock().await = pub_sub_connection;

        Ok(())
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
            pub_sub_connection: Arc::clone(&self.pub_sub_connection),
            redis_config: self.redis_config.clone(),
            instance_id: self.instance_id.clone(),
            service_name: self.service_name.clone(),
            locks: self.locks.clone(),
            presence_task: None,
            message_callbacks: Arc::clone(&self.message_callbacks),
            listener_task: Mutex::new(None),
            connection_status: Arc::clone(&self.connection_status),
        }
    }
}

impl Drop for RedisClient {
    fn drop(&mut self) {
        info!("RedisClient is being dropped");
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            if let Err(e) = self.stop_periodic_tasks() {
                error!("Error stopping periodic tasks: {}", e);
            }
            if !self.locks.is_empty() {
                let mut self_clone = self.clone();
                rt.spawn(async move {
                    let lock_names: Vec<_> = self_clone.locks.keys().cloned().collect();
                    for name in lock_names {
                        if let Err(e) = self_clone.release_lock(&name).await {
                            error!("Error releasing lock {}: {}", name, e);
                        }
                    }
                });
            }
        } else {
            error!("Failed to get current runtime handle in RedisClient::drop");
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
