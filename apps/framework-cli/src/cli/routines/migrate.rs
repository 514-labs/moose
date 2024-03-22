use std::collections::HashMap;
use std::sync::Arc;

use crate::cli::display::Message;
use crate::cli::routines::{Routine, RoutineFailure, RoutineSuccess};
use crate::framework::controller::{
    get_all_framework_objects, FrameworkObjectVersions, SchemaVersion,
};
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

        let path = self
            .project
            .old_version_location(previous_version.as_str())
            .map_err(|err| {
                RoutineFailure::new(
                    Message::new(
                        "Failed".to_string(),
                        "to get old version location".to_string(),
                    ),
                    err,
                )
            })?;

        let mut prev_framework_objects = HashMap::new();
        get_all_framework_objects(&mut prev_framework_objects, &path, &previous_version).map_err(
            |err| {
                RoutineFailure::error(Message::new(
                    "Failed".to_string(),
                    "to get framework objects".to_string(),
                ))
            },
        )?;
        println!("prev_framework_objects: {:?}", prev_framework_objects);

        let mut framework_objects = HashMap::new();
        get_all_framework_objects(
            &mut framework_objects,
            &self.project.schemas_dir(),
            self.project.version(),
        )
        .map_err(|err| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "to get framework objects".to_string(),
            ))
        })?;

        let fo_versions = FrameworkObjectVersions {
            current_version: self.project.version().to_string(),
            current_models: SchemaVersion {
                base_path: self.project.schemas_dir(),
                models: framework_objects,
                typescript_objects: HashMap::new(),
            },
            previous_version_models: HashMap::from([(
                previous_version.to_string(),
                SchemaVersion {
                    base_path: self
                        .project
                        .old_version_location(previous_version.as_str())
                        .unwrap(),
                    models: prev_framework_objects,
                    typescript_objects: HashMap::new(),
                },
            )]),
        };

        let version_syncs = get_all_version_syncs(&self.project, &fo_versions).map_err(|err| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "to generate migrations".to_string(),
            ))
        })?;

        println!("version_syncs: {:?}", version_syncs);

        let new_vs_list = generate_version_syncs(&fo_versions, &version_syncs);
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
