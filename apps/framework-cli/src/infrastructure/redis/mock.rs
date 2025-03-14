use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// An in-memory mock implementation of Redis client functionality.
///
/// This struct provides a fallback implementation when a Redis server is
/// unavailable or for testing purposes. It uses a thread-safe HashMap to
/// simulate Redis storage and implements simplified versions of the Redis
/// operations used by the client.
#[derive(Default, Clone, Debug)]
pub struct MockRedisClient {
    /// Thread-safe in-memory storage that simulates Redis.
    /// Using RwLock instead of Mutex to allow concurrent reads, which
    /// improves performance in read-heavy scenarios common in Redis
    /// usage patterns.
    pub store: Arc<RwLock<HashMap<String, String>>>,
}

impl MockRedisClient {
    /// Creates a new MockRedisClient instance with an empty storage.
    ///
    /// # Returns
    ///
    /// A new MockRedisClient instance with an initialized empty HashMap
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Simulates updating the presence of a service instance.
    ///
    /// This method stores the presence information in the in-memory
    /// HashMap, mimicking the behavior of the real Redis presence update
    /// operation.
    ///
    /// # Parameters
    ///
    /// - `key` - The presence key for the instance
    /// - `value` - The timestamp value to store
    ///
    /// # Returns
    ///
    /// - `Result<()>` - Ok if the operation was successful
    pub async fn update_presence(&self, key: &str, value: u64) -> Result<()> {
        let mut store = self.store.write().await;
        store.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Simulates attempting to acquire a distributed lock.
    ///
    /// This method checks if a lock exists and is owned by the specified
    /// instance. If the lock doesn't exist, it creates it with the
    /// specified instance as owner.
    ///
    /// # Parameters
    ///
    /// - `key` - The lock key
    /// - `instance_id` - The instance ID attempting to acquire the lock
    ///
    /// # Returns
    ///
    /// - `bool` - true if the lock was acquired or already owned, false
    ///   otherwise
    pub async fn attempt_lock(&self, key: &str, instance_id: &str) -> bool {
        let mut store = self.store.write().await;
        if let Some(existing) = store.get(key) {
            return existing == instance_id;
        }
        store.insert(key.to_string(), instance_id.to_string());
        true
    }

    /// Simulates renewing a distributed lock.
    ///
    /// This method checks if the lock exists and is owned by the specified
    /// instance. Unlike real Redis, it doesn't extend TTL as the mock
    /// doesn't implement expiration.
    ///
    /// # Parameters
    ///
    /// - `key` - The lock key
    /// - `instance_id` - The instance ID that should own the lock
    ///
    /// # Returns
    ///
    /// - `bool` - true if the lock exists and is owned by the specified
    ///   instance
    pub async fn renew_lock(&self, key: &str, instance_id: &str) -> bool {
        let store = self.store.read().await;
        if let Some(current) = store.get(key) {
            return current == instance_id;
        }
        false
    }

    /// Simulates releasing a distributed lock.
    ///
    /// This method removes the lock if it exists and is owned by the
    /// specified instance.
    ///
    /// # Parameters
    ///
    /// - `key` - The lock key
    /// - `instance_id` - The instance ID that should own the lock
    ///
    /// # Returns
    ///
    /// - `Result<()>` - Ok if the operation was successful
    pub async fn release_lock(&self, key: &str, instance_id: &str) -> Result<()> {
        let mut store = self.store.write().await;
        if let Some(current) = store.get(key) {
            if current == instance_id {
                store.remove(key);
            }
        }
        Ok(())
    }

    /// Simulates posting a message to a queue.
    ///
    /// This method appends a message to the specified queue, creating the
    /// queue if it doesn't exist. Messages are separated by the '|'
    /// character.
    ///
    /// # Parameters
    ///
    /// - `queue` - The queue name
    /// - `message` - The message to post
    ///
    /// # Returns
    ///
    /// - `Result<()>` - Ok if the operation was successful
    pub async fn post_queue_message(&self, queue: &str, message: &str) -> Result<()> {
        let mut store = self.store.write().await;
        if let Some(existing) = store.get_mut(queue) {
            existing.push('|');
            existing.push_str(message);
        } else {
            store.insert(queue.to_string(), message.to_string());
        }
        Ok(())
    }

    /// Retrieves and removes the first message from a queue.
    ///
    /// This method gets the first message from the specified queue and
    /// removes it, simulating the behavior of Redis RPOPLPUSH command.
    ///
    /// # Parameters
    ///
    /// - `queue` - The queue name
    ///
    /// # Returns
    ///
    /// - `Result<Option<String>>` - The first message in the queue, or
    ///   None if empty
    pub async fn get_queue_message(&self, queue: &str) -> Result<Option<String>> {
        let mut store = self.store.write().await;
        if let Some(existing) = store.get_mut(queue) {
            if let Some(pos) = existing.find('|') {
                let msg = existing[..pos].to_string();
                *existing = existing[pos + 1..].to_string();
                return Ok(Some(msg));
            }

            // If there's no separator but there is content, return the
            // entire content
            if !existing.is_empty() {
                let msg = existing.clone();
                *existing = String::new();
                return Ok(Some(msg));
            }
        }
        Ok(None)
    }
}
