use std::path::PathBuf;

use tokio::process::Child;

use crate::{
    framework::{languages::SupportedLanguages, python, typescript},
    infrastructure::olap::clickhouse::config::ClickHouseConfig,
};

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

    pub fn start(
        &self,
        language: SupportedLanguages,
        clickhouse_config: ClickHouseConfig,
    ) -> Result<Child, AggregationError> {
        match language {
            SupportedLanguages::Typescript => {
                typescript::aggregation::run(clickhouse_config, &self.dir)
            }
            SupportedLanguages::Python => python::aggregation::run(clickhouse_config, &self.dir),
        }
    }
}
