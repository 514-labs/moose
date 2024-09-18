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
use redis::aio::Connection as AsyncConnection;
use redis::AsyncCommands;
pub use redis::RedisError;
use redis::{Client, RedisResult, Script};
use std::env;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use uuid::Uuid;

const REDIS_KEY_PREFIX: &str = "MS"; // MooSe
const KEY_EXPIRATION_TTL: usize = 3; // 3 seconds
const LOCK_TTL: usize = 10; // 10 seconds
const PRESENCE_UPDATE_INTERVAL: u64 = 60; // 1 second
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
    pub async fn new(service_name: &str) -> Result<Self, RedisError> {
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let client = Client::open(redis_url.clone())?;
        let mut connection = client.get_async_connection().await?;
        let pub_sub = client.get_async_connection().await?;
        let instance_id = Uuid::new_v4().to_string();
        let lock_key = format!("{}::{}::leader-lock", REDIS_KEY_PREFIX, service_name);

        println!(
            "Initializing Redis client for {} with instance ID: {}",
            service_name, instance_id
        );

        // Test Redis connection
        match redis::cmd("PING")
            .query_async::<_, String>(&mut connection)
            .await
        {
            Ok(response) => println!("Redis connection successful: {}", response),
            Err(e) => eprintln!("Redis connection failed: {}", e),
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

    pub async fn presence_update(&self) -> RedisResult<()> {
        let key = format!(
            "{}::{}::{}::presence",
            REDIS_KEY_PREFIX, self.service_name, self.instance_id
        );
        let mut conn = self.connection.lock().await;
        let current_value: Option<String> = conn.get(&key).await?;
        println!("Current presence value: {:?}", current_value);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        println!("Updating presence key: {} with value: {}", key, now);
        let result = conn.set_ex(&key, &now, KEY_EXPIRATION_TTL).await;
        println!("Presence update result: {:?}", result);
        result
    }

    pub async fn attempt_leadership(&self) -> RedisResult<bool> {
        let mut conn = self.connection.lock().await;
        let result: bool = conn.set_nx(&self.lock_key, &self.instance_id).await?;
        if result {
            conn.expire(&self.lock_key, LOCK_TTL).await?;
        }

        let mut is_leader = self.is_leader.lock().await;
        *is_leader = result;

        if *is_leader {
            println!("Instance {} became leader", self.instance_id);
            self.renew_lock().await?;
        }

        Ok(*is_leader)
    }

    pub async fn renew_lock(&self) -> RedisResult<bool> {
        let mut conn = self.connection.lock().await;
        let extended: bool = conn.expire(&self.lock_key, LOCK_TTL).await?;

        if !extended {
            println!("Failed to extend leader lock, lost leadership");
            let mut is_leader = self.is_leader.lock().await;
            *is_leader = false;
        }

        Ok(extended)
    }

    pub async fn release_lock(&self) -> RedisResult<()> {
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
            .await?;

        println!("Instance {} released leadership", self.instance_id);
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
    ) -> RedisResult<()> {
        let channel = format!(
            "{}::{}::{}::msgchannel",
            REDIS_KEY_PREFIX, self.service_name, target_instance_id
        );
        let mut conn = self.pub_sub.lock().await;
        let _: () = conn.publish(&channel, message).await?;
        Ok(())
    }

    pub async fn broadcast_message(&self, message: &str) -> RedisResult<()> {
        let channel = format!("{}::{}::msgchannel", REDIS_KEY_PREFIX, self.service_name);
        let mut conn = self.pub_sub.lock().await;
        let _: () = conn.publish(&channel, message).await?;
        Ok(())
    }

    pub async fn get_queue_message(&self) -> RedisResult<Option<String>> {
        let source_queue = format!("{}::{}::mqrecieved", REDIS_KEY_PREFIX, self.service_name);
        let destination_queue = format!("{}::{}::mqprocess", REDIS_KEY_PREFIX, self.service_name);
        let mut conn = self.connection.lock().await;
        conn.rpoplpush(&source_queue, &destination_queue).await
    }

    pub async fn post_queue_message(&self, message: &str) -> RedisResult<()> {
        let queue = format!("{}::{}::mqrecieved", REDIS_KEY_PREFIX, self.service_name);
        let mut conn = self.connection.lock().await;
        let _: () = conn.rpush(&queue, message).await?;
        Ok(())
    }

    pub async fn mark_queue_message(&self, message: &str, success: bool) -> RedisResult<()> {
        let in_progress_queue =
            format!("{}::{}::mqinprogress", REDIS_KEY_PREFIX, self.service_name);
        let incomplete_queue = format!("{}::{}::mqincomplete", REDIS_KEY_PREFIX, self.service_name);

        let mut conn = self.connection.lock().await;
        if success {
            let _: () = conn.lrem(&in_progress_queue, 0, message).await?;
        } else {
            let mut pipeline = redis::pipe();
            pipeline
                .lrem(&in_progress_queue, 0, message)
                .rpush(&incomplete_queue, message);

            pipeline.query_async(&mut *conn).await?;
        }
        Ok(())
    }

    pub async fn start_periodic_tasks(&mut self) {
        println!("Starting periodic tasks");
        let self_ref = Arc::new(self.clone());

        self.presence_task = Some(tokio::spawn(async move {
            let presence_client = Arc::clone(&self_ref);
            println!("Presence update task started");
            let mut interval = interval(Duration::from_secs(PRESENCE_UPDATE_INTERVAL));
            loop {
                println!("Waiting for next tick...");
                interval.tick().await;
                println!("Tick received");
                println!(
                    "Attempting to update presence at {:?}",
                    std::time::SystemTime::now()
                );
                match presence_client.presence_update().await {
                    Ok(_) => println!("Presence updated successfully"),
                    Err(e) => eprintln!("Error updating presence: {}", e),
                }
                println!("Loop iteration completed");
            }
        }));

        let lock_renewal_client = self.clone();
        self.lock_renewal_task = Some(tokio::spawn(async move {
            println!("Lock renewal task started");
            let mut interval = interval(Duration::from_secs(LOCK_RENEWAL_INTERVAL));
            loop {
                println!("Waiting for next lock renewal tick...");
                interval.tick().await;
                println!("Lock renewal tick received");
                if lock_renewal_client.is_current_leader().await {
                    println!("Current instance is leader, renewing lock...");
                    if let Err(e) = lock_renewal_client.renew_lock().await {
                        eprintln!("Error renewing lock: {}", e);
                    }
                } else {
                    println!("Current instance is not leader, attempting leadership...");
                    if let Err(e) = lock_renewal_client.attempt_leadership().await {
                        eprintln!("Error attempting leadership: {}", e);
                    }
                }
            }
        }));
        println!("Periodic tasks started");
    }

    pub async fn stop_periodic_tasks(&mut self) {
        if let Some(task) = self.presence_task.take() {
            task.abort();
        }
        if let Some(task) = self.lock_renewal_task.take() {
            task.abort();
        }
    }

    pub async fn check_connection(&self) -> RedisResult<()> {
        let mut conn = self.connection.lock().await;
        redis::cmd("PING")
            .query_async::<_, String>(&mut *conn)
            .await?;
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
        println!("RedisClient is being dropped");
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            self.stop_periodic_tasks().await;
            if self.is_current_leader().await {
                if let Err(e) = self.release_lock().await {
                    eprintln!("Error releasing lock: {}", e);
                }
            }
        });
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
