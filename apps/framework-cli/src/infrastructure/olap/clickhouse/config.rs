//! # Clickhouse Config
//! Module to handle the creation of the Clickhouse config files
//!
//! ## Suggested Improvements
//! - we need to understand clickhouse configuration better before we can go deep on it's configuration
//!

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClickHouseConfig {
    pub db_name: String, // ex. local
    pub user: String,
    pub password: String,
    pub use_ssl: bool,
    pub host: String,   // ex. localhost
    pub host_port: i32, // ex. 18123
}

impl Default for ClickHouseConfig {
    fn default() -> Self {
        Self {
            db_name: "local".to_string(),
            user: "panda".to_string(),
            password: "pandapass".to_string(),
            use_ssl: false,
            host: "localhost".to_string(),
            host_port: 18123,
        }
    }
}
