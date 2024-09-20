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
//! Note: Make sure to set the REDIS_URL environment variable or the client will default to "redis://127.0.0.1:6379".
use anyhow::{Context, Result};
use log::{error, info};
use redis::aio::Connection as AsyncConnection;
use redis::AsyncCommands;
use redis::{Client, Script};
use std::env;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, timeout, Duration};
use uuid::Uuid;

const REDIS_KEY_PREFIX: &str = "MS"; // MooSe
const KEY_EXPIRATION_TTL: usize = 3; // 3 seconds
const LOCK_TTL: usize = 10; // 10 seconds
const PRESENCE_UPDATE_INTERVAL: u64 = 1; // 1 second
const LOCK_RENEWAL_INTERVAL: u64 = 3; // 3 seconds

pub struct RedisClient {
    connection: Arc<Mutex<AsyncConnection>>,
    pub_sub: Arc<Mutex<AsyncConnection>>,
    service_name: String,
    instance_id: String,
    lock_key: String,
    is_leader: Arc<Mutex<bool>>,
    presence_task: Option<JoinHandle<()>>,
    lock_renewal_task: Option<JoinHandle<()>>,
}

impl RedisClient {
    pub async fn new(service_name: &str) -> Result<Self> {
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let client = Client::open(redis_url.clone()).context("Failed to create Redis client")?;
        let mut connection = client
            .get_async_connection()
            .await
            .context("Failed to establish Redis connection")?;
        let pub_sub = client
            .get_async_connection()
            .await
            .context("Failed to establish Redis pub/sub connection")?;
        let instance_id = Uuid::new_v4().to_string();
        let lock_key = format!("{}::{}::leader-lock", REDIS_KEY_PREFIX, service_name);

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
            service_name: service_name.to_string(),
            instance_id,
            lock_key,
            is_leader: Arc::new(Mutex::new(false)),
            presence_task: None,
            lock_renewal_task: None,
        })
    }

    pub async fn presence_update(&self) -> Result<()> {
        let key = format!(
            "{}::{}::{}::presence",
            REDIS_KEY_PREFIX, self.service_name, self.instance_id
        );
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get current time")?
            .as_secs()
            .to_string();
        let result = {
            let mut conn = self.connection.lock().await;
            conn.set_ex(&key, &now, KEY_EXPIRATION_TTL).await
        };

        result.context("Failed to update presence")
    }

    pub async fn attempt_leadership(&self) -> Result<bool> {
        let mut conn = self.connection.lock().await;
        let result: bool = conn
            .set_nx(&self.lock_key, &self.instance_id)
            .await
            .context("Failed to attempt leadership")?;
        if result {
            conn.expire(&self.lock_key, LOCK_TTL)
                .await
                .context("Failed to set expiration on leadership lock")?;
        }

        let mut is_leader = self.is_leader.lock().await;
        *is_leader = result;

        if *is_leader {
            info!("Instance {} became leader", self.instance_id);
            self.renew_lock().await?;
        }

        Ok(*is_leader)
    }

    pub async fn renew_lock(&self) -> Result<bool> {
        let mut conn = self.connection.lock().await;
        let extended: bool = conn
            .expire(&self.lock_key, LOCK_TTL)
            .await
            .context("Failed to renew leadership lock")?;

        if !extended {
            info!("Failed to extend leader lock, lost leadership");
            let mut is_leader = self.is_leader.lock().await;
            *is_leader = false;
        }

        Ok(extended)
    }

    pub async fn release_lock(&self) -> Result<()> {
        let script = Script::new(
            r"
            if redis.call('get', KEYS[1]) == ARGV[1] then
                return redis.call('del', KEYS[1])
            else
                return 0
            end
        ",
        );

        let mut conn = self.connection.lock().await;
        let _: () = script
            .key(&self.lock_key)
            .arg(&self.instance_id)
            .invoke_async(&mut *conn)
            .await
            .context("Failed to release leadership lock")?;

        info!("Instance {} released leadership", self.instance_id);
        let mut is_leader = self.is_leader.lock().await;
        *is_leader = false;

        Ok(())
    }

    pub async fn is_current_leader(&self) -> bool {
        *self.is_leader.lock().await
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
            REDIS_KEY_PREFIX, self.service_name, target_instance_id
        );
        let mut conn = self.pub_sub.lock().await;
        let _: () = conn
            .publish(&channel, message)
            .await
            .context("Failed to send message to instance")?;
        Ok(())
    }

    pub async fn broadcast_message(&self, message: &str) -> Result<()> {
        let channel = format!("{}::{}::msgchannel", REDIS_KEY_PREFIX, self.service_name);
        let mut conn = self.pub_sub.lock().await;
        let _: () = conn
            .publish(&channel, message)
            .await
            .context("Failed to broadcast message")?;
        Ok(())
    }

    pub async fn get_queue_message(&self) -> Result<Option<String>> {
        let source_queue = format!("{}::{}::mqrecieved", REDIS_KEY_PREFIX, self.service_name);
        let destination_queue = format!("{}::{}::mqprocess", REDIS_KEY_PREFIX, self.service_name);
        let mut conn = self.connection.lock().await;
        conn.rpoplpush(&source_queue, &destination_queue)
            .await
            .context("Failed to get queue message")
    }

    pub async fn post_queue_message(&self, message: &str) -> Result<()> {
        let queue = format!("{}::{}::mqrecieved", REDIS_KEY_PREFIX, self.service_name);
        let mut conn = self.connection.lock().await;
        let _: () = conn
            .rpush(&queue, message)
            .await
            .context("Failed to post queue message")?;
        Ok(())
    }

    pub async fn mark_queue_message(&self, message: &str, success: bool) -> Result<()> {
        let in_progress_queue =
            format!("{}::{}::mqinprogress", REDIS_KEY_PREFIX, self.service_name);
        let incomplete_queue = format!("{}::{}::mqincomplete", REDIS_KEY_PREFIX, self.service_name);

        let mut conn = self.connection.lock().await;
        if success {
            let _: () = conn
                .lrem(&in_progress_queue, 0, message)
                .await
                .context("Failed to mark queue message as successful")?;
        } else {
            let mut pipeline = redis::pipe();
            pipeline
                .lrem(&in_progress_queue, 0, message)
                .rpush(&incomplete_queue, message);

            pipeline
                .query_async(&mut *conn)
                .await
                .context("Failed to mark queue message as incomplete")?;
        }
        Ok(())
    }

    pub async fn start_periodic_tasks(&mut self) {
        info!("Starting periodic tasks");
        let self_ref = Arc::new(self.clone());

        let presence_client = Arc::clone(&self_ref);
        self.presence_task = Some(tokio::spawn(async move {
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                async move {
                    info!("Presence update task started");
                    let mut interval = interval(Duration::from_secs(PRESENCE_UPDATE_INTERVAL));
                    loop {
                        interval.tick().await;
                        match timeout(Duration::from_secs(10), presence_client.presence_update())
                            .await
                        {
                            Ok(result) => match result {
                                Ok(_) => info!("Presence updated successfully"),
                                Err(e) => error!("Error updating presence: {}", e),
                            },
                            Err(_) => error!("Presence update timed out"),
                        }
                        info!("Presence update loop iteration completed");
                        tokio::task::yield_now().await; // Yield control back to the runtime
                    }
                }
            })) {
                error!("Presence task panicked: {:?}", e);
            }
        }));

        let lock_renewal_client = Arc::clone(&self_ref);
        self.lock_renewal_task = Some(tokio::spawn(async move {
            info!("Lock renewal task started");
            let mut interval = interval(Duration::from_secs(LOCK_RENEWAL_INTERVAL));
            loop {
                interval.tick().await;
                if lock_renewal_client.is_current_leader().await {
                    match lock_renewal_client.renew_lock().await {
                        Ok(_) => info!("Lock renewed successfully"),
                        Err(e) => {
                            error!("Error renewing lock: {}", e);
                            // Add a delay before retrying to avoid tight loop on error
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                } else {
                    match lock_renewal_client.attempt_leadership().await {
                        Ok(_) => info!("Leadership attempt completed"),
                        Err(e) => {
                            error!("Error attempting leadership: {}", e);
                            // Add a delay before retrying to avoid tight loop on error
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
                info!("Lock renewal loop iteration completed");
                tokio::task::yield_now().await; // Yield control back to the runtime
            }
        }));

        info!("Periodic tasks started");
    }

    pub async fn stop_periodic_tasks(&mut self) -> Result<()> {
        if let Some(task) = self.presence_task.take() {
            task.abort();
        }
        if let Some(task) = self.lock_renewal_task.take() {
            task.abort();
        }
        Ok(())
    }

    pub async fn check_connection(&self) -> Result<()> {
        let mut conn = self.connection.lock().await;
        redis::cmd("PING")
            .query_async::<_, String>(&mut *conn)
            .await
            .context("Failed to ping Redis server")?;
        Ok(())
    }
}

impl Clone for RedisClient {
    fn clone(&self) -> Self {
        Self {
            connection: self.connection.clone(),
            pub_sub: self.pub_sub.clone(),
            service_name: self.service_name.clone(),
            instance_id: self.instance_id.clone(),
            lock_key: self.lock_key.clone(),
            is_leader: self.is_leader.clone(),
            presence_task: None,
            lock_renewal_task: None,
        }
    }
}

impl Drop for RedisClient {
    fn drop(&mut self) {
        info!("RedisClient is being dropped");
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            let mut self_clone = self.clone(); // Ensure self_clone is mutable
            rt.spawn(async move {
                if let Err(e) = self_clone.stop_periodic_tasks().await {
                    error!("Error stopping periodic tasks: {}", e);
                }
                if self_clone.is_current_leader().await {
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
    #[tokio::test]
    async fn test_redis_operations() {
        let client = RedisClient::new("test_service")
            .await
            .expect("Failed to create Redis client");

        // Test set and get
        let mut conn = client.connection.lock().await;
        let _: () = conn
            .set("test_key", "test_value")
            .await
            .expect("Failed to set value");
        let result: Option<String> = conn.get("test_key").await.expect("Failed to get value");
        assert_eq!(result, Some("test_value".to_string()));

        // Test delete
        let _: () = conn.del("test_key").await.expect("Failed to delete key");
        let result: Option<String> = conn
            .get("test_key")
            .await
            .expect("Failed to get value after deletion");
        assert_eq!(result, None);

        // Test set_ex
        let _: () = conn
            .set_ex("test_ex_key", "test_ex_value", 1)
            .await
            .expect("Failed to set value with expiry");
        let result: Option<String> = conn
            .get("test_ex_key")
            .await
            .expect("Failed to get value with expiry");
        assert_eq!(result, Some("test_ex_value".to_string()));
        tokio::time::sleep(Duration::from_secs(2)).await;
        let result: Option<String> = conn
            .get("test_ex_key")
            .await
            .expect("Failed to get expired value");
        assert_eq!(result, None);

        // Test rpush and rpoplpush
        let _: () = conn
            .rpush("source_list", "item1")
            .await
            .expect("Failed to push to list");
        let _: () = conn
            .rpush("source_list", "item2")
            .await
            .expect("Failed to push to list");
        let result: Option<String> = conn
            .rpoplpush("source_list", "dest_list")
            .await
            .expect("Failed to rpoplpush");
        assert_eq!(result, Some("item2".to_string()));

        // Test lrem
        let _: () = conn
            .lrem("dest_list", 0, "item2")
            .await
            .expect("Failed to remove from list");
        let result: Option<String> = conn
            .rpoplpush("dest_list", "source_list")
            .await
            .expect("Failed to rpoplpush after lrem");
        assert_eq!(result, None);

        // Cleanup
        let _: () = conn
            .del("dest_list")
            .await
            .expect("Failed to delete dest_list");
        let _: () = conn
            .del("source_list")
            .await
            .expect("Failed to delete source_list");
    }
}
