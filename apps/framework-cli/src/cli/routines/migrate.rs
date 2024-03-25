use std::sync::Arc;

use crate::cli::display::Message;
use crate::cli::routines::{crawl_schema, Routine, RoutineFailure, RoutineSuccess};
use crate::infrastructure::olap::clickhouse::version_sync::{
    generate_version_syncs, get_all_version_syncs, parse_version,
};
use crate::project::Project;

pub struct GenerateMigration {
    project: Arc<Project>,
}
impl GenerateMigration {
    pub fn new(project: Arc<Project>) -> Self {
        Self { project }
    }
}

impl Routine for GenerateMigration {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let previous_version = match self
            .project
            .supported_old_versions
            .keys()
            .max_by_key(|v| parse_version(v))
        {
            None => {
                return Ok(RoutineSuccess::info(Message {
                    action: "No".to_string(),
                    details: "old versions found".to_string(),
                }))
            }
            Some(previous_version) => previous_version,
        };

        let fo_versions =
            crawl_schema(&self.project, &self.project.versions_sorted()).map_err(|err| {
                RoutineFailure::new(
                    Message::new("Failed".to_string(), "to crawl schema".to_string()),
                    err,
                )
            })?;

        let version_syncs = get_all_version_syncs(&self.project, &fo_versions).map_err(|err| {
            RoutineFailure::new(
                Message::new("Failed".to_string(), "to generate migrations".to_string()),
                err,
            )
        })?;

        println!("version_syncs: {:?}", version_syncs);

        let new_vs_list = generate_version_syncs(&fo_versions, &version_syncs, previous_version);
        print!("{:?}", new_vs_list);

        let flow_dir = self.project.flows_dir();
        for vs in new_vs_list {
            let file_path = flow_dir.join(format!(
                "{}_migrate__{}__{}.sql",
                vs.model_name,
                vs.source_version.replace('.', "_"),
                vs.dest_version.replace('.', "_")
            ));
            let file_content = vs.migration_function;
            std::fs::write(&file_path, file_content).map_err(|err| {
                RoutineFailure::new(
                    Message::new("Failed".to_string(), "to write migration file".to_string()),
                    err,
                )
            })?;
        }

        Ok(RoutineSuccess::success(Message {
            action: "Generated".to_string(),
            details: "migrations".to_string(),
        }))
    }
}
