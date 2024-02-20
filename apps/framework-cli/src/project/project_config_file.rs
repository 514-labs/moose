use serde::{Deserialize, Serialize};

use crate::cli::local_webserver::LocalWebserverConfig;
use crate::framework::languages::SupportedLanguages;
use crate::infrastructure::console::ConsoleConfig;
use crate::infrastructure::olap::clickhouse::config::ClickhouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectConfigFile {
    pub language: SupportedLanguages,
    pub redpanda: RedpandaConfig,
    pub clickhouse: ClickhouseConfig,
    pub http_server: LocalWebserverConfig,
    pub console: ConsoleConfig,
}

impl Default for ProjectConfigFile {
    fn default() -> Self {
        Self {
            language: SupportedLanguages::Typescript,
            redpanda: RedpandaConfig::default(),
            clickhouse: ClickhouseConfig::default(),
            http_server: LocalWebserverConfig::default(),
            console: ConsoleConfig::default(),
        }
    }
}
