use anyhow::Context;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex as TokioMutex;

/// Manager for service instance presence tracking.
///
/// This struct provides methods to register and maintain the presence
/// of a service instance in a distributed system using Redis TTL keys.
/// It is cloneable to allow sharing across multiple tasks.
#[derive(Clone)]
pub struct PresenceManager {
    /// Unique identifier for this service instance.
    pub instance_id: String,
    /// Prefix for Redis keys to prevent collisions.
    pub key_prefix: String,
}

impl PresenceManager {
    /// Creates a new PresenceManager instance.
    ///
    /// # Parameters
    ///
    /// - `instance_id` - A unique identifier for this service instance
    /// - `key_prefix` - Prefix for Redis keys to prevent collisions
    ///
    /// # Returns
    ///
    /// A new PresenceManager instance configured with the provided
    /// parameters
    pub fn new(instance_id: String, key_prefix: String) -> Self {
        PresenceManager {
            instance_id,
            key_prefix,
        }
    }

    /// Generates the Redis key used for tracking this instance's presence.
    ///
    /// The key follows the format: `{key_prefix}::{instance_id}::presence`
    ///
    /// # Returns
    ///
    /// A string containing the formatted presence key
    fn presence_key(&self) -> String {
        format!("{}::{}::presence", self.key_prefix, self.instance_id)
    }

    /// Updates the instance's presence in Redis with a new TTL.
    ///
    /// This method:
    /// 1. Generates the presence key for this instance
    /// 2. Gets the current timestamp
    /// 3. Sets the key in Redis with the timestamp as value
    /// 4. Applies a TTL of 10 seconds to the key
    ///
    /// This should be called periodically (typically every few seconds)
    /// to maintain the instance's presence. If the instance fails to
    /// update its presence, the key will expire after the TTL.
    ///
    /// # Parameters
    ///
    /// - `conn` - Redis connection manager
    ///
    /// # Returns
    ///
    /// - `anyhow::Result<()>` - Ok(()) if the operation was successful
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to get the current time
    /// - Redis operation fails
    pub async fn update_presence(
        &self,
        conn: Arc<TokioMutex<ConnectionManager>>,
    ) -> anyhow::Result<()> {
        let key = self.presence_key();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("Failed to get current time")?
            .as_secs();
        {
            let mut conn_guard = conn.lock().await;
            let _: () = redis::AsyncCommands::set_ex(&mut *conn_guard, &key, now, 10).await?;
        }
        Ok(())
    }
}
