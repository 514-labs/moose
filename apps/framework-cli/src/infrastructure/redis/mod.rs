pub mod connection;
pub mod connection_pool;
pub mod leadership;
pub mod lua_scripts;
pub mod messaging;
pub mod mock;
pub mod presence;
pub mod redis_client;

// Re-export the main client for convenience
pub use redis_client::RedisClient;
pub use redis_client::RedisConfig;
pub use redis_client::ThreadSafeRedisClient;
