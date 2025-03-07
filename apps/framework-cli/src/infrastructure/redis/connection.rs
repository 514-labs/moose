use futures::FutureExt;
use redis::aio::ConnectionManager;
use redis::{Client, RedisError};
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex as TokioMutex;
use tokio::time;

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

/// A thread-safe wrapper around Redis ConnectionManager.
///
/// This struct provides a wrapper around the Redis ConnectionManager that
/// ensures thread safety and adds features like connection health
/// monitoring and automatic reconnection with exponential backoff.
#[derive(Clone)]
pub struct ConnectionManagerWrapper {
    /// The main connection used for regular Redis commands.
    pub connection: Arc<TokioMutex<ConnectionManager>>,
    /// A separate connection dedicated to pub/sub operations.
    pub pub_sub: Arc<TokioMutex<ConnectionManager>>,
    /// Atomic flag indicating the connection state (true = connected).
    pub state: Arc<AtomicBool>, // true = connected
}

impl ConnectionManagerWrapper {
    /// Creates a new ConnectionManagerWrapper with the provided
    /// Redis client.
    ///
    /// This method establishes two separate connections to Redis:
    /// 1. A main connection for regular Redis commands
    /// 2. A dedicated connection for pub/sub operations
    ///
    /// Both connections are wrapped in Tokio mutexes to ensure thread
    /// safety. The connection state is initially set to connected (true).
    ///
    /// # Parameters
    ///
    /// - `client` - A reference to a Redis Client instance used to
    ///    establish connections
    ///
    /// # Returns
    ///
    /// - `Result<Self, RedisError>` - A new ConnectionManagerWrapper or a
    ///   Redis error
    ///
    /// # Errors
    ///
    /// This method will return an error if it fails to establish either
    /// connection to Redis.
    pub async fn new(client: &Client) -> Result<Self, RedisError> {
        // Create connection manager for normal operations
        let conn = client.get_connection_manager().await?;
        let pub_sub_conn = client.get_connection_manager().await?;
        Ok(ConnectionManagerWrapper {
            connection: Arc::new(TokioMutex::new(conn)),
            pub_sub: Arc::new(TokioMutex::new(pub_sub_conn)),
            state: Arc::new(AtomicBool::new(true)),
        })
    }

    /// Checks the health of the Redis connection by sending a PING command.
    ///
    /// This method attempts to send a PING command to Redis with a timeout
    /// of 2 seconds. If the PING succeeds, it returns true. If it fails or
    /// times out, it updates the connection state to disconnected (false)
    /// and returns false.
    ///
    /// # Returns
    ///
    /// - `bool` - true if the connection is healthy, false otherwise
    pub async fn ping(&self) -> bool {
        let timeout_future = time::timeout(
            Duration::from_secs(2),
            AssertUnwindSafe(async {
                redis::cmd("PING")
                    .query_async::<_, String>(&mut *self.connection.lock().await)
                    .await
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

    /// Attempts to reconnect to Redis with exponential backoff.
    ///
    /// This method is called when a connection failure is detected. It
    /// attempts to reestablish the connection with an exponential backoff
    /// strategy:
    ///
    /// 1. Initial backoff of 5 seconds
    /// 2. Doubles the backoff time after each failed attempt
    /// 3. Caps the maximum backoff at 60 seconds
    ///
    /// Once a connection is successfully established, it updates the
    /// connection state to connected (true) and returns.
    ///
    /// # Parameters
    ///
    /// - `client` - A reference to a Redis Client instance used to
    ///   establish a new connection
    /// - `_config` - A reference to the Redis configuration
    ///   (currently unused)
    pub async fn attempt_reconnection(&self, client: &Client, _config: &RedisConfig) {
        let mut backoff = 5;
        while !self.state.load(Ordering::SeqCst) {
            time::sleep(Duration::from_secs(backoff)).await;
            if let Ok(new_conn) = client.get_connection_manager().await {
                let mut conn_guard = self.connection.lock().await;
                *conn_guard = new_conn;
                self.state.store(true, Ordering::SeqCst);
                break;
            } else {
                backoff = std::cmp::min(backoff * 2, 60);
            }
        }
    }
}
