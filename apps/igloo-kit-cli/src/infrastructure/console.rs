use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsoleConfig {
    pub host_port: i32, // ex. 18123
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            host_port: ConsoleConfig::PORT,
        }
    }
}

impl ConsoleConfig {
    pub const PORT: i32 = 3001;
}
