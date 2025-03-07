use redis::aio::ConnectionManager;
use redis::Script;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use super::redis_client::RedisConfig;

/// Manager for distributed leadership and lock coordination.
///
/// This struct provides methods to acquire, verify, renew, and release
/// distributed locks across multiple service instances using Redis.
pub struct LeadershipManager {
    /// Unique identifier for this service instance.
    pub instance_id: String,
    /// Redis configuration for connecting to the Redis server.
    pub redis_config: RedisConfig,
}

impl LeadershipManager {
    /// Creates a new LeadershipManager instance.
    ///
    /// # Parameters
    ///
    /// - `instance_id` - A unique identifier for this service instance
    /// - `config` - Redis configuration for connecting to the Redis server
    ///
    /// # Returns
    ///
    /// A new LeadershipManager instance configured with the provided
    /// parameters
    pub fn new(instance_id: String, config: RedisConfig) -> Self {
        LeadershipManager {
            instance_id,
            redis_config: config,
        }
    }

    /// Attempts to acquire a distributed lock.
    ///
    /// This method tries to acquire a lock with the following logic:
    /// - If the lock doesn't exist, it's created with this instance as the owner
    /// - If the lock exists and is owned by this instance, the TTL is refreshed
    /// - If the lock exists and is owned by another instance, acquisition fails
    ///
    /// The operation is performed atomically using a Lua script.
    ///
    /// # Parameters
    ///
    /// - `conn` - Redis connection manager
    /// - `lock_key` - Unique identifier for the lock
    /// - `ttl` - Time To Live in seconds for the lock
    ///
    /// # Returns
    ///
    /// - `bool` - true if the lock was acquired or already owned, false
    ///   otherwise
    pub async fn attempt_lock(
        &self,
        conn: Arc<TokioMutex<ConnectionManager>>,
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
            let mut conn_guard = conn.lock().await;
            script
                .key(lock_key)
                .arg(instance)
                .arg(ttl)
                .invoke_async::<_, i32>(&mut *conn_guard)
                .await
        };
        matches!(result, Ok(val) if val == 1)
    }

    /// Renews an existing lock by extending its TTL.
    ///
    /// This method extends the TTL of a lock only if it's currently owned
    /// by the specified instance. The operation is performed atomically
    /// using a Lua script.
    ///
    /// # Parameters
    ///
    /// - `conn` - Redis connection manager
    /// - `lock_key` - Unique identifier for the lock
    /// - `instance_id` - The instance ID that should own the lock
    /// - `ttl` - New Time To Live in seconds for the lock
    ///
    /// # Returns
    ///
    /// - `anyhow::Result<bool>` - Ok(true) if the lock was renewed,
    ///   Ok(false) if not owned by this instance
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis operation fails
    pub async fn renew_lock(
        &self,
        conn: Arc<TokioMutex<ConnectionManager>>,
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
            .invoke_async::<_, i32>(&mut *conn.lock().await)
            .await?;
        Ok(result == 1)
    }

    /// Releases a lock if it's owned by the specified instance.
    ///
    /// This method deletes a lock only if it's currently owned by the
    /// specified instance. The operation is performed atomically using
    /// a Lua script.
    ///
    /// # Parameters
    ///
    /// - `conn` - Redis connection manager
    /// - `lock_key` - Unique identifier for the lock
    /// - `instance_id` - The instance ID that should own the lock
    ///
    /// # Returns
    ///
    /// - `anyhow::Result<()>` - Ok(()) if the operation was successful
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis operation fails
    pub async fn release_lock(
        &self,
        conn: Arc<TokioMutex<ConnectionManager>>,
        lock_key: &str,
        instance_id: &str,
    ) -> anyhow::Result<()> {
        let script = redis::Script::new(
            "if redis.call('GET', KEYS[1]) == ARGV[1] then\n return redis.call('DEL', KEYS[1])\nelse\n return 0\nend"
        );
        let _result: () = script
            .key(lock_key)
            .arg(instance_id)
            .invoke_async(&mut *conn.lock().await)
            .await?;
        Ok(())
    }

    /// Checks if a lock is currently owned by the specified instance.
    ///
    /// This method verifies if a lock exists and is owned by the specified
    /// instance without modifying the lock. The operation is performed
    /// atomically using a Lua script.
    ///
    /// # Parameters
    ///
    /// - `conn` - Redis connection manager
    /// - `lock_key` - Unique identifier for the lock
    /// - `instance_id` - The instance ID to check ownership against
    ///
    /// # Returns
    ///
    /// - `anyhow::Result<bool>` - Ok(true) if the lock is owned by the
    ///   instance, Ok(false) otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if the Redis operation fails
    pub async fn has_lock(
        &self,
        conn: Arc<TokioMutex<ConnectionManager>>,
        lock_key: &str,
        instance_id: &str,
    ) -> anyhow::Result<bool> {
        let script = redis::Script::new(
            "if redis.call('GET', KEYS[1]) == ARGV[1] then return 1 else return 0 end",
        );
        let result: i32 = script
            .key(lock_key)
            .arg(instance_id)
            .invoke_async::<_, i32>(&mut *conn.lock().await)
            .await?;
        Ok(result == 1)
    }
}
