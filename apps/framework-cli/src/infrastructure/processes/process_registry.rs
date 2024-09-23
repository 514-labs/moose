use crate::project::Project;

use super::aggregations_registry::AggregationProcessRegistry;
use super::consumption_registry::ConsumptionProcessRegistry;
use super::functions_registry::FunctionProcessRegistry;

pub struct ProcessRegistries {
    pub functions: FunctionProcessRegistry,
    pub aggregations: AggregationProcessRegistry,
    pub blocks: AggregationProcessRegistry,
    pub consumption: ConsumptionProcessRegistry,
}

impl ProcessRegistries {
    pub fn new(project: &Project) -> Self {
        let functions = FunctionProcessRegistry::new(
            project.redpanda_config.clone(),
            project.project_location.clone(),
        );
        let aggregations = AggregationProcessRegistry::new(
            project.language,
            project.aggregations_dir(),
            project.project_location.clone(),
            project.clickhouse_config.clone(),
            true,
        );
        let blocks = AggregationProcessRegistry::new(
            project.language,
            project.blocks_dir(),
            project.project_location.clone(),
            project.clickhouse_config.clone(),
            false,
        );
        let consumption = ConsumptionProcessRegistry::new(
            project.language,
            project.clickhouse_config.clone(),
            project.consumption_dir(),
            project.project_location.clone(),
        );

        Self {
            functions,
            aggregations,
            blocks,
            consumption,
        }
    }
}
