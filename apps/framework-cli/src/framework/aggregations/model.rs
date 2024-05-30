use std::path::PathBuf;

use tokio::process::Child;

use crate::{framework::typescript, infrastructure::olap::clickhouse::config::ClickHouseConfig};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AggregationError {
    #[error("Failed to start/stop the aggregation process")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Aggregation {
    pub dir: PathBuf,
}

impl Aggregation {
    pub fn id(&self) -> String {
        "onlyone".to_string()
    }

    pub fn start(&self, clickhouse_config: ClickHouseConfig) -> Result<Child, AggregationError> {
        typescript::aggregation::run(clickhouse_config, &self.dir)
    }
}
