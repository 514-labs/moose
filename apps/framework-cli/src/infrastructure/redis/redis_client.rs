//! Redis Client Module
//!
//! This module provides a Redis client implementation with support for leader election,
//! presence updates, and message passing (Sending) and message queuing.
//!
//! # Example Usage
//!
//! ```rust
//! use your_crate_name::infrastructure::redis::RedisClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the Redis client
//!     let mut client = RedisClient::new("my_service").await?;
//!
//!     // Start periodic tasks (presence updates and lock renewal)
//!     client.start_periodic_tasks().await;
//!
//!     // Attempt to gain leadership
//!     let is_leader = client.attempt_leadership().await?;
//!     println!("Is leader: {}", is_leader);
//!
//!     // Send a message to another instance
//!     client.send_message_to_instance("Hello", "target_instance_id").await?;
//!
//!     // Broadcast a message to all instances
//!     client.broadcast_message("Broadcast message").await?;
//!
//!     // Post a message to the queue
//!     client.post_queue_message("New task").await?;
//!
//!     // Get a message from the queue
//!     if let Some(message) = client.get_queue_message().await? {
//!         println!("Received message: {}", message);
//!         // Process the message...
//!         client.mark_queue_message(&message, true).await?;
//!     }
//!
//!     // The client will automatically stop periodic tasks and release the lock (if leader)
//!     // when it goes out of scope due to the Drop implementation
//!
//!     Ok(())
//! }
//! ```
//!
//! Note: Make sure to set the MOOSE_REDIS_URL environment variable or the client will default to "redis://127.0.0.1:6379".
use anyhow::{Context, Result};
use log::{error, info};
use redis::aio::Connection as RedisConnection;
use redis::{AsyncCommands, Client, RedisError, Script};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use tokio_stream::StreamExt;
use uuid::Uuid;

// Internal constants that we don't expose to the user
const KEY_EXPIRATION_TTL: u64 = 3; // 3 seconds
const PRESENCE_UPDATE_INTERVAL: u64 = 1; // 1 second

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
    connection: Arc<Mutex<RedisConnection>>,
    pub_sub: Arc<Mutex<RedisConnection>>,
    redis_config: RedisConfig,
    service_name: String,
    instance_id: String,
    locks: HashMap<String, RedisLock>,
    presence_task: Option<JoinHandle<()>>,
    message_callbacks: Arc<Mutex<Vec<MessageCallback>>>,
    listener_task: Mutex<Option<JoinHandle<()>>>,
}

impl RedisClient {
    pub async fn new(service_name: String, redis_config: RedisConfig) -> Result<Self> {
        let client =
            Client::open(redis_config.url.clone()).context("Failed to create Redis client")?;

        let mut connection = client
            .get_async_connection()
            .await
            .context("Failed to establish Redis connection")?;

        let pub_sub = client
            .get_async_connection()
            .await
            .context("Failed to establish Redis pub/sub connection")?;

        let instance_id = Uuid::new_v4().to_string();

        info!(
            "Initializing Redis client for {} with instance ID: {}",
            service_name, instance_id
        );

        // Test Redis connection
        match redis::cmd("PING")
            .query_async::<_, String>(&mut connection)
            .await
        {
            Ok(response) => info!("Redis connection successful: {}", response),
            Err(e) => error!("Redis connection failed: {}", e),
        }

        let client = Self {
            connection: Arc::new(Mutex::new(connection)),
            pub_sub: Arc::new(Mutex::new(pub_sub)),
            redis_config,
            instance_id,
            service_name,
            locks: HashMap::new(),
            presence_task: None,
            message_callbacks: Arc::new(Mutex::new(Vec::new())),
            listener_task: Mutex::new(None),
        };

        // Start the message listener as part of initialization
        match client.start_message_listener().await {
            Ok(_) => info!("Successfully started message listener"),
            Err(e) => error!("Failed to start message listener: {}", e),
        }

        info!(
            "<RedisClient> Started {} with id {}",
            client.get_service_name(),
            client.get_instance_id()
        );

        Ok(client)
    }

    pub async fn presence_update(&mut self) -> Result<()> {
        let key = format!(
            "{}::{}::{}::presence",
            self.redis_config.key_prefix, self.service_name, self.instance_id
        );

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get current time")?
            .as_secs()
            .to_string();

        self.connection
            .lock()
            .await
            .set_ex(&key, &now, KEY_EXPIRATION_TTL)
            .await
            .context("Failed to update presence")
    }

    pub async fn register_lock(&mut self, name: &str, ttl: i64) -> Result<()> {
        let lock_key = format!(
            "{}::{}::{}::lock",
            self.redis_config.key_prefix, self.service_name, name
        );
        let lock = RedisLock { key: lock_key, ttl };
        self.locks.insert(name.to_string(), lock);
        Ok(())
    }

    pub async fn attempt_lock(&self, name: &str) -> Result<bool> {
        if let Some(lock) = self.locks.get(name) {
            let result: bool = self
                .connection
                .lock()
                .await
                .set_nx(&lock.key, &self.instance_id)
                .await
                .context("Failed to attempt lock")?;

            if result {
                let _: () = self
                    .connection
                    .lock()
                    .await
                    .expire(&lock.key, lock.ttl)
                    .await
                    .context("Failed to set expiration on lock")?;
            }

            Ok(result)
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

    pub async fn release_lock(&self, name: &str) -> Result<()> {
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

            Ok(())
        } else {
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

            Ok(value == Some(self.instance_id.clone()))
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
        let channel = format!(
            "{}::{}::{}::msgchannel",
            self.redis_config.key_prefix, self.service_name, target_instance_id
        );
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
        let channel = format!(
            "{}::{}::msgchannel",
            self.redis_config.key_prefix, self.service_name
        );
        let _: () = self
            .pub_sub
            .lock()
            .await
            .publish(&channel, message)
            .await
            .context("Failed to broadcast message")?;
        Ok(())
    }

    pub async fn get_queue_message(&self) -> Result<Option<String>> {
        let source_queue = format!(
            "{}::{}::mqrecieved",
            self.redis_config.key_prefix, self.service_name
        );
        let destination_queue = format!(
            "{}::{}::mqprocess",
            self.redis_config.key_prefix, self.service_name
        );
        self.connection
            .lock()
            .await
            .rpoplpush(&source_queue, &destination_queue)
            .await
            .context("Failed to get queue message")
    }

    pub async fn post_queue_message(&self, message: &str) -> Result<()> {
        let queue = format!(
            "{}::{}::mqrecieved",
            self.redis_config.key_prefix, self.service_name
        );
        let _: () = self
            .connection
            .lock()
            .await
            .rpush(&queue, message)
            .await
            .context("Failed to post queue message")?;
        Ok(())
    }

    pub async fn mark_queue_message(&mut self, message: &str, success: bool) -> Result<()> {
        let in_progress_queue = format!(
            "{}::{}::mqinprogress",
            self.redis_config.key_prefix, self.service_name
        );
        let incomplete_queue = format!(
            "{}::{}::mqincomplete",
            self.redis_config.key_prefix, self.service_name
        );

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
        info!("Starting periodic tasks");

        self.presence_task = Some(tokio::spawn({
            let mut presence_client = self.clone();
            async move {
                let mut interval = interval(Duration::from_secs(PRESENCE_UPDATE_INTERVAL));
                loop {
                    interval.tick().await;
                    if let Err(e) = presence_client.presence_update().await {
                        error!("Error updating presence: {}", e);
                    }
                }
            }
        }));

        info!("Periodic tasks started");
    }

    pub fn stop_periodic_tasks(&mut self) -> Result<()> {
        if let Some(task) = self.presence_task.take() {
            task.abort();
        }
        Ok(())
    }

    pub async fn check_connection(&mut self) -> Result<()> {
        redis::cmd("PING")
            .query_async::<_, String>(&mut *self.connection.lock().await)
            .await
            .context("Failed to ping Redis server")?;
        Ok(())
    }

    pub async fn register_message_handler(&self, callback: Arc<dyn Fn(String) + Send + Sync>) {
        self.message_callbacks
            .lock()
            .await
            .push(Arc::new(move |message: String| {
                let callback = Arc::clone(&callback);
                Box::pin(async move {
                    (callback)(message);
                }) as Pin<Box<dyn Future<Output = ()> + Send>>
            }));
    }

    async fn start_message_listener(&self) -> Result<(), RedisError> {
        let instance_channel = format!(
            "{}::{}::{}::msgchannel",
            self.redis_config.key_prefix, self.service_name, self.instance_id
        );
        let broadcast_channel = format!(
            "{}::{}::msgchannel",
            self.redis_config.key_prefix, self.service_name
        );

        info!(
            "<RedisClient> Listening for messages on channels: {} and {}",
            instance_channel, broadcast_channel
        );

        // Create a separate PubSub connection for listening to messages
        let client = Client::open(self.redis_config.url.clone())?;
        let pubsub_conn = client.get_async_connection().await?;
        let mut pubsub = pubsub_conn.into_pubsub();

        pubsub
            .subscribe(&[&instance_channel, &broadcast_channel])
            .await
            .map_err(|e| {
                error!("<RedisClient> Failed to subscribe to channels: {}", e);
                e
            })?;

        let callback = self.message_callbacks.clone();

        let listener_task = tokio::spawn(async move {
            let mut pubsub_stream = pubsub.on_message();

            while let Some(msg) = pubsub_stream.next().await {
                if let Ok(payload) = msg.get_payload::<String>() {
                    info!("<RedisClient> Received pubsub message: {}", payload);
                    let callbacks = callback.lock().await.clone();
                    for cb in callbacks {
                        cb(payload.clone()).await;
                    }
                }
            }
        });

        // Store the listener task
        *self.listener_task.lock().await = Some(listener_task);

        Ok(())
    }
}

impl Clone for RedisClient {
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
        }
    }
}

impl Drop for RedisClient {
    fn drop(&mut self) {
        info!("RedisClient is being dropped");
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            let mut self_clone = self.clone();
            rt.spawn(async move {
                if let Err(e) = self_clone.stop_periodic_tasks() {
                    error!("Error stopping periodic tasks: {}", e);
                }
                let lock_names: Vec<_> = self_clone.locks.keys().cloned().collect();
                for name in lock_names {
                    if let Err(e) = self_clone.release_lock(&name).await {
                        error!("Error releasing lock {}: {}", name, e);
                    }
                }
            });
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
