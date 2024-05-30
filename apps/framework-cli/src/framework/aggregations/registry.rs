use std::collections::HashMap;

use log::info;
use tokio::process::Child;

use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;

use super::model::{Aggregation, AggregationError};

pub struct AggregationProcessRegistry {
    registry: HashMap<String, Child>,
    clickhouse_config: ClickHouseConfig,
}

impl AggregationProcessRegistry {
    pub fn new(clickhouse_config: ClickHouseConfig) -> Self {
        Self {
            registry: HashMap::new(),
            clickhouse_config,
        }
    }

    pub fn start_all(&mut self, aggregation: Aggregation) -> Result<(), AggregationError> {
        info!("Starting aggregation {:?}...", aggregation);
        let child = aggregation.start(self.clickhouse_config.clone())?;

        self.registry.insert(aggregation.id(), child);

        Ok(())
    }

    pub async fn stop_all(&mut self) -> Result<(), AggregationError> {
        info!("Stopping aggregation...");

        for child in self.registry.values_mut() {
            child.kill().await?;
        }

        self.registry.clear();

        Ok(())
    }
}
