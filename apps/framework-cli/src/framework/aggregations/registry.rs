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
    #[allow(dead_code)]
    features: Features,
}

impl AggregationProcessRegistry {
    pub fn new(
        language: SupportedLanguages,
        clickhouse_config: ClickHouseConfig,
        features: Features,
    ) -> Self {
        Self {
            registry: HashMap::new(),
            language,
            clickhouse_config,
            features,
        }
    }

    pub fn start(&mut self, aggregation: Aggregation) -> Result<(), AggregationError> {
        info!("Starting aggregation {:?}...", aggregation);
        let child = aggregation.start(self.language, self.clickhouse_config.clone())?;

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
}
