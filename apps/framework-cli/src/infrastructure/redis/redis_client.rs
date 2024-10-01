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
use crate::utilities::constants::{CONTEXT, CTX_IS_LEADER};
use anyhow::{Context, Result};
use log::{error, info};
use redis::aio::Connection as AsyncConnection;
use redis::AsyncCommands;
use redis::{Client, Script};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use uuid::Uuid;

use lazy_static::lazy_static;
use std::collections::HashMap;

// Wrap CONTEXT in a Mutex
lazy_static! {
    static ref MUTEX_CONTEXT: Mutex<HashMap<String, String>> = Mutex::new(CONTEXT.clone());
}

// Internal constants that we don't expose to the user
const KEY_EXPIRATION_TTL: u64 = 3; // 3 seconds
const LOCK_TTL: i64 = 10; // 10 seconds
const PRESENCE_UPDATE_INTERVAL: u64 = 1; // 1 second
const LOCK_RENEWAL_INTERVAL: u64 = 3; // 3 seconds

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

pub struct RedisClient {
    connection: Arc<Mutex<AsyncConnection>>,
    pub_sub: Arc<Mutex<AsyncConnection>>,
    redis_config: RedisConfig,
    service_name: String,
    instance_id: String,
    lock_key: String,
    is_leader: bool,
    presence_task: Option<JoinHandle<()>>,
    lock_renewal_task: Option<JoinHandle<()>>,
    leadership_callback: Option<Arc<Mutex<dyn Fn() + Send + Sync>>>, // Added this line
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
        let lock_key = format!("{}::{}::leader-lock", redis_config.key_prefix, service_name);

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

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            pub_sub: Arc::new(Mutex::new(pub_sub)),
            redis_config,
            instance_id,
            service_name,
            lock_key,
            is_leader: false,
            presence_task: None,
            lock_renewal_task: None,
            leadership_callback: None, // Added this line
        })
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

    pub async fn attempt_leadership(&mut self) -> Result<bool> {
        let lock_key = self.lock_key.clone();
        let instance_id = self.instance_id.clone();

        info!("Attempting leadership for {}", self.instance_id);
        info!("First getting redis connection lock");

        let result: bool = self
            .connection
            .lock()
            .await
            .set_nx(&lock_key, &instance_id)
            .await
            .context("Failed to attempt leadership")?;

        if result {
            let _: () = self
                .connection
                .lock()
                .await
                .expire(&lock_key, LOCK_TTL)
                .await
                .context("Failed to set expiration on leadership lock")?;
        }

        self.is_leader = result;

        if self.is_leader {
            info!("Instance {} became leader", self.instance_id);
            self.renew_lock().await?;

            let mut context = MUTEX_CONTEXT.lock().await;
            context.insert(CTX_IS_LEADER.to_string(), "true".to_string());

            // Call the leadership callback if set
            if let Some(callback) = &self.leadership_callback {
                let callback = callback.clone();
                tokio::spawn(async move {
                    (callback.lock().await)();
                });
            }
        } else {
            let mut context = MUTEX_CONTEXT.lock().await;
            context.insert(CTX_IS_LEADER.to_string(), "false".to_string());
        }

        Ok(self.is_leader)
    }

    pub async fn renew_lock(&mut self) -> Result<bool> {
        let extended: bool = self
            .connection
            .lock()
            .await
            .expire(&self.lock_key, LOCK_TTL)
            .await
            .context("Failed to renew leadership lock")?;

        if !extended {
            info!("Failed to extend leader lock, lost leadership");
            self.is_leader = false;
        }

        Ok(extended)
    }

    pub async fn release_lock(&mut self) -> Result<()> {
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
            .key(&self.lock_key)
            .arg(&self.instance_id)
            .invoke_async(&mut *self.connection.lock().await)
            .await
            .context("Failed to release leadership lock")?;

        info!("Instance {} released leadership", self.instance_id);
        self.is_leader = false;

        Ok(())
    }

    pub fn is_current_leader(&self) -> bool {
        self.is_leader
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

        self.lock_renewal_task = Some(tokio::spawn({
            let mut lock_renewal_client = self.clone();
            async move {
                let mut interval = interval(Duration::from_secs(LOCK_RENEWAL_INTERVAL));

                // First attempt to gain leadership
                match lock_renewal_client.attempt_leadership().await {
                    Ok(is_leader) => {
                        if is_leader {
                            info!("Successfully gained leadership");
                        } else {
                            info!("Failed to gain leadership initially");
                        }
                    }
                    Err(e) => error!("Error attempting leadership: {}", e),
                }

                loop {
                    interval.tick().await;
                    if lock_renewal_client.is_current_leader() {
                        if let Err(e) = lock_renewal_client.renew_lock().await {
                            error!("Error renewing lock: {}", e);
                        }
                    } else {
                        // Attempt to gain leadership if not currently the leader
                        if let Err(e) = lock_renewal_client.attempt_leadership().await {
                            error!("Error attempting leadership: {}", e);
                        }
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
        if let Some(task) = self.lock_renewal_task.take() {
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

    // Added this method
    pub fn set_leadership_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.leadership_callback = Some(Arc::new(Mutex::new(callback)));
    }
}

impl Clone for RedisClient {
    fn clone(&self) -> Self {
        Self {
            connection: Arc::clone(&self.connection),
            pub_sub: Arc::clone(&self.pub_sub),
            redis_config: self.redis_config.clone(),
            instance_id: self.instance_id.clone(),
            lock_key: self.lock_key.clone(),
            is_leader: self.is_leader,
            service_name: self.service_name.clone(),
            presence_task: None,
            lock_renewal_task: None,
            leadership_callback: self.leadership_callback.clone(), // Added this line
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
                if self_clone.is_current_leader() {
                    if let Err(e) = self_clone.release_lock().await {
                        error!("Error releasing lock: {}", e);
                    }
                }
            });
        } else {
            error!("Failed to get current runtime handle in RedisClient::drop");
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
