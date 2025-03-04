// This file implements a new, modular Redis client as per the proposal in scratch-pr-proposal.md.
use futures::FutureExt;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time;

// -------------------------------
// Redis configuration
// -------------------------------

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

// -------------------------------
// Connection Module
// -------------------------------

pub mod connection {
    use super::*;

    pub enum ConnectionState {
        Connected,
        Disconnected,
        Reconnecting,
    }

    pub struct ConnectionManagerWrapper {
        pub connection: Arc<RwLock<ConnectionManager>>,
        pub pub_sub: Arc<RwLock<ConnectionManager>>,
        pub state: Arc<AtomicBool>, // true = connected
    }

    impl ConnectionManagerWrapper {
        pub async fn new(client: &Client) -> Result<Self, RedisError> {
            // Create connection manager for normal operations
            let conn = client.get_tokio_connection_manager().await?;
            let pub_sub_conn = client.get_tokio_connection_manager().await?;
            Ok(ConnectionManagerWrapper {
                connection: Arc::new(RwLock::new(conn)),
                pub_sub: Arc::new(RwLock::new(pub_sub_conn)),
                state: Arc::new(AtomicBool::new(true)),
            })
        }

        // A non-blocking ping with timeout and catch_unwind
        pub async fn ping(&self) -> bool {
            let conn_lock = self.connection.read().await;
            let ping_future = redis::cmd("PING").query_async::<_, String>(&*conn_lock);
            let timeout = time::timeout(
                Duration::from_secs(2),
                AssertUnwindSafe(ping_future).catch_unwind(),
            );
            match timeout.await {
                Ok(Ok(Ok(_response))) => true,
                _ => {
                    self.state.store(false, Ordering::SeqCst);
                    false
                }
            }
        }

        // Reconnection logic with exponential backoff
        pub async fn attempt_reconnection(&self, client: &Client, _config: &RedisConfig) {
            let mut backoff = 5;
            while !self.state.load(Ordering::SeqCst) {
                time::sleep(Duration::from_secs(backoff)).await;
                if let Ok(new_conn) = client.get_tokio_connection_manager().await {
                    let mut write_conn = self.connection.write().await;
                    *write_conn = new_conn;
                    self.state.store(true, Ordering::SeqCst);
                    break;
                } else {
                    backoff = std::cmp::min(backoff * 2, 60);
                }
            }
        }
    }
}

// -------------------------------
// Leadership Module
// -------------------------------

pub mod leadership {
    use super::*;
    use redis::Script;

    pub struct LeadershipManager {
        pub instance_id: String,
        pub redis_config: RedisConfig,
    }

    impl LeadershipManager {
        pub fn new(instance_id: String, config: RedisConfig) -> Self {
            LeadershipManager {
                instance_id,
                redis_config: config,
            }
        }

        // Attempt to acquire a leadership lock using a Lua script
        pub async fn attempt_lock(
            &self,
            conn: Arc<RwLock<ConnectionManager>>,
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
                let mut conn_guard = conn.write().await;
                script
                    .key(lock_key)
                    .arg(instance)
                    .arg(ttl)
                    .invoke_async::<_, i32>(&mut *conn_guard)
                    .await
            };
            matches!(result, Ok(val) if val == 1)
        }
    }
}

// -------------------------------
// Presence Module
// -------------------------------

pub mod presence {
    use super::*;
    use anyhow::Context;

    pub struct PresenceManager {
        pub instance_id: String,
        pub key_prefix: String,
    }

    impl PresenceManager {
        pub fn new(instance_id: String, key_prefix: String) -> Self {
            PresenceManager {
                instance_id,
                key_prefix,
            }
        }

        fn presence_key(&self) -> String {
            format!("{}::{}::presence", self.key_prefix, self.instance_id)
        }

        // Update presence using a TTL key
        pub async fn update_presence(
            &self,
            conn: Arc<RwLock<ConnectionManager>>,
        ) -> anyhow::Result<()> {
            let key = self.presence_key();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context("Failed to get current time")?
                .as_secs();
            {
                let mut conn_guard = conn.write().await;
                conn_guard.set_ex(&key, now, 10).await?;
            }
            Ok(())
        }
    }
}

// -------------------------------
// Messaging Module
// -------------------------------

pub mod messaging {
    use super::*;

    pub struct MessagingManager {
        pub key_prefix: String,
    }

    impl MessagingManager {
        pub fn new(key_prefix: String) -> Self {
            MessagingManager { key_prefix }
        }

        pub fn channel_for(&self, target: &str) -> String {
            format!("{}::{}::msgchannel", self.key_prefix, target)
        }

        // Publish a message to a channel
        pub async fn publish_message(
            &self,
            conn: Arc<RwLock<ConnectionManager>>,
            target: &str,
            message: &str,
        ) -> anyhow::Result<()> {
            let channel = self.channel_for(target);
            let mut conn_guard = conn.write().await;
            conn_guard.publish(&channel, message).await?;
            Ok(())
        }
    }
}

// -------------------------------
// Core API: NewRedisClient
// -------------------------------

pub struct NewRedisClient {
    pub config: RedisConfig,
    pub service_name: String,
    pub instance_id: String,
    pub connection_manager: connection::ConnectionManagerWrapper,
    pub leadership_manager: leadership::LeadershipManager,
    pub presence_manager: presence::PresenceManager,
    pub messaging_manager: messaging::MessagingManager,
}

impl NewRedisClient {
    pub async fn new(service_name: String, config: RedisConfig) -> anyhow::Result<Self> {
        let client = Client::open(config.url.clone())?;
        let connection_manager = connection::ConnectionManagerWrapper::new(&client).await?;

        // Determine instance_id, using HOSTNAME env or generate a UUID
        let instance_id =
            std::env::var("HOSTNAME").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

        let leadership_manager =
            leadership::LeadershipManager::new(instance_id.clone(), config.clone());
        let presence_manager =
            presence::PresenceManager::new(instance_id.clone(), config.key_prefix.clone());
        let messaging_manager = messaging::MessagingManager::new(config.key_prefix.clone());

        Ok(NewRedisClient {
            config,
            service_name,
            instance_id,
            connection_manager,
            leadership_manager,
            presence_manager,
            messaging_manager,
        })
    }

    // Update presence
    pub async fn update_presence(&self) -> anyhow::Result<()> {
        self.presence_manager
            .update_presence(self.connection_manager.connection.clone())
            .await
    }

    // Try to acquire leadership lock
    pub async fn try_acquire_leadership(&self, ttl: i64) -> bool {
        let lock_key = format!("{}::leadership::lock", self.config.key_prefix);
        self.leadership_manager
            .attempt_lock(self.connection_manager.connection.clone(), &lock_key, ttl)
            .await
    }

    // Publish a message
    pub async fn publish_message(&self, target: &str, message: &str) -> anyhow::Result<()> {
        self.messaging_manager
            .publish_message(self.connection_manager.pub_sub.clone(), target, message)
            .await
    }

    // Check connection health
    pub async fn check_connection(&self) -> bool {
        self.connection_manager.ping().await
    }

    // Ensure connection is alive; if not, attempt reconnection
    pub async fn ensure_connection(&self) -> anyhow::Result<()> {
        if !self.connection_manager.state.load(Ordering::SeqCst) {
            let client = Client::open(self.config.url.clone())?;
            self.connection_manager
                .attempt_reconnection(&client, &self.config)
                .await;
        }
        Ok(())
    }
}
