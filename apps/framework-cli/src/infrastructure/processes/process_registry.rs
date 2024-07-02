use crate::cli::settings::Features;
use crate::framework::aggregations::registry::AggregationProcessRegistry;
use crate::framework::consumption::registry::ConsumptionProcessRegistry;
use crate::framework::languages::SupportedLanguages;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::stream::redpanda::RedpandaConfig;

use super::functions_registry::FunctionProcessRegistry;

pub struct ProcessRegistries {
    pub flows: FunctionProcessRegistry,
    pub aggregations: AggregationProcessRegistry,
    pub consumption: ConsumptionProcessRegistry,
}

impl ProcessRegistries {
    pub fn new(
        kafka_config: RedpandaConfig,
        language: SupportedLanguages,
        clickhouse_config: ClickHouseConfig,
        features: &Features,
    ) -> Self {
        let flows = FunctionProcessRegistry::new(kafka_config.clone());
        let aggregations =
            AggregationProcessRegistry::new(language, clickhouse_config.clone(), features);
        let consumption = ConsumptionProcessRegistry::new(language, clickhouse_config);

        Self {
            flows,
            aggregations,
            consumption,
        }
    }
}
