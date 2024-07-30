use std::{collections::HashMap, path::PathBuf};

use log::info;
use tokio::process::Child;

use crate::{
    framework::{
        aggregations::model::AggregationError, core::infrastructure::olap_process::OlapProcess,
        languages::SupportedLanguages, python, typescript,
    },
    infrastructure::olap::clickhouse::config::ClickHouseConfig,
};

// to be changed to BlocksProcessRegistry and remove is_aggregation
pub struct AggregationProcessRegistry {
    registry: HashMap<String, Child>,
    language: SupportedLanguages,
    dir: PathBuf,
    clickhouse_config: ClickHouseConfig,
    is_aggregation: bool,
}

impl AggregationProcessRegistry {
    pub fn new(
        language: SupportedLanguages,
        dir: PathBuf,
        clickhouse_config: ClickHouseConfig,
        is_aggregation: bool,
    ) -> Self {
        Self {
            registry: HashMap::new(),
            language,
            dir,
            clickhouse_config,
            is_aggregation,
        }
    }

    pub fn start(&mut self, olap_process: &OlapProcess) -> Result<(), AggregationError> {
        info!("Starting aggregation {:?}...", olap_process);

        let child = match self.language {
            SupportedLanguages::Typescript => typescript::aggregation::run(
                self.clickhouse_config.clone(),
                &self.dir,
                !self.is_aggregation,
            )?,
            SupportedLanguages::Python => python::aggregation::run(
                self.clickhouse_config.clone(),
                &self.dir,
                !self.is_aggregation,
            )?,
        };

        self.registry.insert(olap_process.id(), child);

        Ok(())
    }

    pub async fn stop(&mut self, _olap_process: &OlapProcess) -> Result<(), AggregationError> {
        info!("Stopping aggregation...");

        for child in self.registry.values_mut() {
            child.kill().await?;
        }

        self.registry.clear();

        Ok(())
    }
}
