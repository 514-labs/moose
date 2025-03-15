use futures::future::{BoxFuture, FutureExt};
use redis::aio::ConnectionManager;
use redis::{Client, RedisError};
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use super::connection_pool::ConnectionPool;
use super::redis_client::RedisConfig;

/// Represents the possible states of a Redis connection.
///
/// This enum is used to track and communicate the current state of the
/// connection to Redis, which is useful for monitoring and diagnostics.
pub enum ConnectionState {
    /// The connection to Redis is established and operational.
    Connected,
    /// The connection to Redis has been lost or closed.
    Disconnected,
    /// The system is currently attempting to reestablish a lost connection.
    Reconnecting,
}

/// Wrapper around Redis ConnectionManager with additional functionality
///
/// This wrapper provides:
/// 1. Connection state tracking
/// 2. Automatic reconnection capabilities
/// 3. Separate connections for regular commands and pub/sub
/// 4. Connection pooling for better performance under load
#[derive(Clone)]
pub struct ConnectionManagerWrapper {
    pub connection: ConnectionManager,
    pub pub_sub: ConnectionManager,
    pub state: Arc<AtomicBool>,
    pub connection_pool: Option<ConnectionPool>,
}

impl ConnectionManagerWrapper {
    /// Creates a new ConnectionManagerWrapper
    ///
    /// # Arguments
    /// * `client` - Redis client to create connection managers from
    ///
    /// # Returns
    /// * `Result<Self, RedisError>` - New wrapper or error
    pub async fn new(client: &Client) -> Result<Self, RedisError> {
        // Create connection manager for normal operations
        let conn = client.get_connection_manager().await?;
        let pub_sub_conn = client.get_connection_manager().await?;

        Ok(ConnectionManagerWrapper {
            connection: conn,
            pub_sub: pub_sub_conn,
            state: Arc::new(AtomicBool::new(true)),
            connection_pool: None,
        })
    }

    /// Creates a new ConnectionManagerWrapper with a connection pool
    ///
    /// # Arguments
    /// * `client` - Redis client to create connection managers from
    /// * `pool_size` - Number of connections to maintain in the pool
    ///
    /// # Returns
    /// * `Result<Self, RedisError>` - New wrapper or error
    pub async fn new_with_pool(client: &Client, pool_size: usize) -> Result<Self, anyhow::Error> {
        // Create connection manager for normal operations
        let conn = client.get_connection_manager().await?;
        let pub_sub_conn = client.get_connection_manager().await?;

        // Create connection pool
        let redis_url = match &client.get_connection_info().addr {
            redis::ConnectionAddr::Tcp(host, port) => format!("redis://{}:{}", host, port),
            redis::ConnectionAddr::TcpTls { host, port, .. } => {
                format!("rediss://{}:{}", host, port)
            }
            redis::ConnectionAddr::Unix(path) => format!("unix://{}", path.display()),
        };
        let pool = ConnectionPool::new(&redis_url, pool_size).await?;

        Ok(ConnectionManagerWrapper {
            connection: conn,
            pub_sub: pub_sub_conn,
            state: Arc::new(AtomicBool::new(true)),
            connection_pool: Some(pool),
        })
    }

    /// Checks if the connection is alive by sending a PING command
    ///
    /// # Returns
    /// * `bool` - True if connection is alive, false otherwise
    pub async fn ping(&self) -> bool {
        let timeout_future = timeout(
            Duration::from_secs(2),
            AssertUnwindSafe(async {
                let mut conn = self.connection.clone();
                redis::cmd("PING").query_async::<_, String>(&mut conn).await
            })
            .catch_unwind(),
        );

        match timeout_future.await {
            Ok(Ok(Ok(_response))) => true,
            _ => {
                self.state.store(false, Ordering::SeqCst);
                false
            }
        }
    }

    /// Attempts to reconnect to Redis with exponential backoff
    ///
    /// # Arguments
    /// * `client` - Redis client to reconnect with
    /// * `config` - Redis configuration
    pub async fn attempt_reconnection(&self, client: &Client, _config: &RedisConfig) {
        let mut backoff = 5;
        while !self.state.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_secs(backoff)).await;
            if let Ok(new_conn) = client.get_connection_manager().await {
                // Update our connection with the new one
                // This is safe because ConnectionManager is designed for concurrent use
                // and we're replacing the entire instance
                unsafe {
                    let self_ptr =
                        self as *const ConnectionManagerWrapper as *mut ConnectionManagerWrapper;
                    (*self_ptr).connection = new_conn;
                }
                self.state.store(true, Ordering::SeqCst);
                break;
            }
            backoff = std::cmp::min(backoff * 2, 60);
        }
    }

    /// Executes a Redis command using the connection manager
    ///
    /// If a connection pool is available, it will use a connection from the pool.
    /// Otherwise, it will use the default connection.
    ///
    /// # Arguments
    /// * `f` - Function that takes a ConnectionManager and returns a Future
    ///
    /// # Returns
    /// * `redis::RedisResult<R>` - Result of the operation
    pub async fn execute<F, R: 'static>(&self, f: F) -> redis::RedisResult<R>
    where
        F: for<'a> FnOnce(&'a mut ConnectionManager) -> BoxFuture<'a, redis::RedisResult<R>>
            + Send
            + Clone,
    {
        // If we have a connection pool, use it
        if let Some(pool) = &self.connection_pool {
            // Instead of trying to use the pool's execute method directly,
            // we'll get a connection and use it manually
            let mut conn = pool.get_connection();

            // Call the function with the connection
            let future = f(&mut conn);

            // Await the future
            return future.await;
        }

        // If we don't have a connection pool, use the direct connection
        // Clone the connection to get a mutable version
        let mut conn = self.connection.clone();
        f(&mut conn).await
    }

    /// Executes a Redis pub/sub command using the pub/sub connection manager
    ///
    /// # Arguments
    /// * `f` - Function that takes a ConnectionManager and returns a Future
    ///
    /// # Returns
    /// * `redis::RedisResult<R>` - Result of the operation
    pub async fn execute_pubsub<F, R>(&self, f: F) -> redis::RedisResult<R>
    where
        F: for<'a> FnOnce(&'a mut ConnectionManager) -> BoxFuture<'a, redis::RedisResult<R>> + Send,
    {
        let mut conn = self.pub_sub.clone();
        f(&mut conn).await
    }
}
