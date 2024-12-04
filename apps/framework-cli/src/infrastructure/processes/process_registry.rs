use crate::project::Project;

use super::blocks_registry::BlocksProcessRegistry;
use super::consumption_registry::ConsumptionProcessRegistry;
use super::functions_registry::FunctionProcessRegistry;

pub struct ProcessRegistries {
    pub functions: FunctionProcessRegistry,
    pub blocks: BlocksProcessRegistry,
    pub consumption: ConsumptionProcessRegistry,
}

impl ProcessRegistries {
    pub fn new(project: &Project) -> Self {
        let functions = FunctionProcessRegistry::new(
            project.redpanda_config.clone(),
            project.project_location.clone(),
        );
        let blocks = BlocksProcessRegistry::new(
            project.language,
            project.blocks_dir(),
            project.project_location.clone(),
            project.clickhouse_config.clone(),
        );
        let consumption = ConsumptionProcessRegistry::new(
            project.language,
            project.clickhouse_config.clone(),
            project.jwt.clone(),
            project.consumption_dir(),
            project.project_location.clone(),
        );

        Self {
            functions,
            blocks,
            consumption,
        }
    }
}
