//! Redis Connection Pool
//!
//! This module provides a connection pool for Redis operations, allowing
//! for better distribution of load across multiple connections.

use anyhow::Result;
use redis::aio::ConnectionManager;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A pool of Redis connections that can be used for distributing load
/// across multiple connections.
///
/// The pool maintains a fixed number of connection managers that can be
/// cloned and used for Redis operations. It uses a simple round-robin
/// strategy to distribute operations across connections.
#[derive(Clone)]
pub struct ConnectionPool {
    connections: Vec<ConnectionManager>,
    next_index: std::sync::Arc<AtomicUsize>,
}

impl ConnectionPool {
    /// Creates a new connection pool with the specified number of connections.
    ///
    /// # Arguments
    ///
    /// * `redis_url` - The Redis URL to connect to
    /// * `pool_size` - The number of connections to maintain in the pool
    ///
    /// # Returns
    ///
    /// A Result containing the new ConnectionPool or an error
    pub async fn new(redis_url: &str, pool_size: usize) -> Result<Self> {
        let mut connections = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            let client = redis::Client::open(redis_url)?;
            let connection = ConnectionManager::new(client).await?;
            connections.push(connection);
        }

        Ok(Self {
            connections,
            next_index: std::sync::Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Gets the next connection from the pool using round-robin distribution.
    ///
    /// # Returns
    ///
    /// A cloned ConnectionManager that can be used for Redis operations
    pub fn get_connection(&self) -> ConnectionManager {
        let index = self.next_index.fetch_add(1, Ordering::SeqCst) % self.connections.len();
        self.connections[index].clone()
    }

    /// Executes a Redis operation using a connection from the pool.
    ///
    /// # Arguments
    ///
    /// * `f` - A function that takes a ConnectionManager and returns a Future
    ///
    /// # Returns
    ///
    /// The result of the operation
    pub async fn execute<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(ConnectionManager) -> futures::future::BoxFuture<'static, Result<R, E>>,
    {
        let conn = self.get_connection();
        f(conn).await
    }

    /// Returns the size of the connection pool.
    ///
    /// # Returns
    ///
    /// The number of connections in the pool
    pub fn pool_size(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use redis::AsyncCommands;

    #[tokio::test]
    async fn test_connection_pool() -> Result<()> {
        // This test requires a running Redis instance
        // Skip if REDIS_URL is not set
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());

        let pool = ConnectionPool::new(&redis_url, 3).await?;
        assert_eq!(pool.pool_size(), 3);

        // Test that we can execute operations
        for i in 0..10 {
            let key = format!("test_key_{}", i);
            let value = format!("test_value_{}", i);

            pool.execute(|mut conn| {
                let key = key.clone();
                let value = value.clone();
                Box::pin(async move {
                    conn.set::<_, _, ()>(&key, &value).await?;
                    let result: String = conn.get(&key).await?;
                    assert_eq!(result, value);
                    conn.del::<_, ()>(&key).await?;
                    Ok::<_, redis::RedisError>(())
                })
            })
            .await?;
        }

        Ok(())
    }
}
