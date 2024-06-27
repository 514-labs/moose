use std::collections::HashMap;

use log::info;
use tokio::process::Child;

use crate::{
    cli::settings::Features, framework::languages::SupportedLanguages,
    infrastructure::olap::clickhouse::config::ClickHouseConfig,
};

use super::model::{Aggregation, AggregationError};

pub struct AggregationProcessRegistry {
    registry: HashMap<String, Child>,
    language: SupportedLanguages,
    clickhouse_config: ClickHouseConfig,
    features: Features,
}

impl AggregationProcessRegistry {
    pub fn new(
        language: SupportedLanguages,
        clickhouse_config: ClickHouseConfig,
        features: &Features,
    ) -> Self {
        Self {
            registry: HashMap::new(),
            language,
            clickhouse_config,
            features: features.clone(),
        }
    }

    pub fn start(&mut self, aggregation: Aggregation) -> Result<(), AggregationError> {
        info!("Starting aggregation {:?}...", aggregation);
        let child = aggregation.start(
            self.language,
            self.clickhouse_config.clone(),
            self.features.blocks,
        )?;

        self.registry.insert(aggregation.id(), child);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), AggregationError> {
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
