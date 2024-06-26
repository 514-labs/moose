use std::collections::HashMap;

use log::info;
use tokio::process::Child;

use crate::{
    framework::languages::SupportedLanguages,
    infrastructure::olap::clickhouse::config::ClickHouseConfig, utilities::system::kill_child,
};

use super::model::{Consumption, ConsumptionError};

pub struct ConsumptionProcessRegistry {
    registry: HashMap<String, Child>,
    clickhouse_config: ClickHouseConfig,
    language: SupportedLanguages,
}

impl ConsumptionProcessRegistry {
    pub fn new(language: SupportedLanguages, clickhouse_config: ClickHouseConfig) -> Self {
        Self {
            registry: HashMap::new(),
            language,
            clickhouse_config,
        }
    }

    pub fn start(&mut self, consumption: Consumption) -> Result<(), ConsumptionError> {
        info!("Starting consumption {:?}...", consumption);
        let child = consumption.start(self.language, self.clickhouse_config.clone())?;

        self.registry.insert(consumption.id(), child);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), ConsumptionError> {
        info!("Stopping consumption...");

        for child in self.registry.values_mut() {
            kill_child(child).await?;
        }

        self.registry.clear();

        Ok(())
    }
}
