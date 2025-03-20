use crate::cli::settings::Settings;
use crate::project::Project;

use super::blocks_registry::BlocksProcessRegistry;
use super::consumption_registry::ConsumptionProcessRegistry;
use super::functions_registry::FunctionProcessRegistry;
use super::orchestration_workers_registry::OrchestrationWorkersRegistry;

pub struct ProcessRegistries {
    pub functions: FunctionProcessRegistry,
    pub blocks: BlocksProcessRegistry,
    pub consumption: ConsumptionProcessRegistry,
    pub orchestration_workers: OrchestrationWorkersRegistry,
}

impl ProcessRegistries {
    pub fn new(project: &Project, settings: &Settings) -> Self {
        let functions = FunctionProcessRegistry::new(project.clone());
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
            project.clone(),
        );

        let orchestration_workers = OrchestrationWorkersRegistry::new(project, settings);

        Self {
            functions,
            blocks,
            consumption,
            orchestration_workers,
        }
    }
}
