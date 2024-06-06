use std::path::PathBuf;

use tokio::process::Child;

use crate::{
    framework::typescript, infrastructure::olap::clickhouse::config::ClickHouseConfig,
    utilities::system::KillProcessError,
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConsumptionError {
    #[error("Failed to start/stop the consumption process")]
    IoError(#[from] std::io::Error),

    #[error("Kill process Error")]
    KillProcessError(#[from] KillProcessError),
}

#[derive(Debug, Clone)]
pub struct Consumption {
    pub dir: PathBuf,
}

impl Consumption {
    pub fn id(&self) -> String {
        "onlyone".to_string()
    }

    pub fn start(&self, clickhouse_config: ClickHouseConfig) -> Result<Child, ConsumptionError> {
        typescript::consumption::run(clickhouse_config, &self.dir)
    }
}
