use crate::cli::settings::Features;
use crate::framework::consumption::registry::ConsumptionProcessRegistry;
use crate::project::Project;

use super::aggregations_registry::AggregationProcessRegistry;
use super::functions_registry::FunctionProcessRegistry;

pub struct ProcessRegistries {
    pub functions: FunctionProcessRegistry,
    pub aggregations: AggregationProcessRegistry,
    pub consumption: ConsumptionProcessRegistry,
}

impl ProcessRegistries {
    pub fn new(project: &Project, features: &Features) -> Self {
        let functions = FunctionProcessRegistry::new(project.redpanda_config.clone());
        let aggs_dir = if features.blocks {
            project.blocks_dir()
        } else {
            project.aggregations_dir()
        };
        let aggregations = AggregationProcessRegistry::new(
            project.language,
            aggs_dir,
            project.clickhouse_config.clone(),
            features,
        );
        let consumption =
            ConsumptionProcessRegistry::new(project.language, project.clickhouse_config.clone());

        Self {
            functions,
            aggregations,
            consumption,
        }
    }
}
