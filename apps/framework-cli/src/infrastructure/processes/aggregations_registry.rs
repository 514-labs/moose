use std::{collections::HashMap, path::PathBuf};

use log::info;
use tokio::process::Child;

use crate::{
    cli::settings::Features,
    framework::{
        aggregations::model::AggregationError, core::infrastructure::olap_process::OlapProcess,
        languages::SupportedLanguages, python, typescript,
    },
    infrastructure::olap::clickhouse::config::ClickHouseConfig,
};

pub struct AggregationProcessRegistry {
    registry: HashMap<String, Child>,
    language: SupportedLanguages,
    dir: PathBuf,
    clickhouse_config: ClickHouseConfig,
    features: Features,
}

impl AggregationProcessRegistry {
    pub fn new(
        language: SupportedLanguages,
        dir: PathBuf,
        clickhouse_config: ClickHouseConfig,
        features: &Features,
    ) -> Self {
        Self {
            registry: HashMap::new(),
            language,
            dir,
            clickhouse_config,
            features: features.clone(),
        }
    }

    pub fn start(&mut self, olap_process: &OlapProcess) -> Result<(), AggregationError> {
        info!("Starting aggregation {:?}...", olap_process);

        let child = match self.language {
            SupportedLanguages::Typescript => typescript::aggregation::run(
                self.clickhouse_config.clone(),
                &self.dir,
                self.features.blocks,
            )?,
            SupportedLanguages::Python => python::aggregation::run(
                self.clickhouse_config.clone(),
                &self.dir,
                self.features.blocks,
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

    pub fn is_blocks_enabled(&self) -> bool {
        self.features.blocks
    }
}
