use std::collections::HashMap;
use std::sync::Arc;

use crate::cli::display::Message;
use crate::cli::routines::{Routine, RoutineFailure, RoutineSuccess};
use crate::framework::controller::get_all_framework_objects;
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
        let framework_objects = match self
            .project
            .supported_old_versions
            .keys()
            .max_by_key(|v| parse_version(v))
        {
            None => {
                return Ok(RoutineSuccess::success(Message {
                    action: "No".to_string(),
                    details: "old versions found".to_string(),
                }))
            }
            Some(previous_version) => {
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

                let mut framework_objects = HashMap::new();
                get_all_framework_objects(&mut framework_objects, &path, &previous_version)
                    .map_err(|err| {
                        RoutineFailure::error(Message::new(
                            "Failed".to_string(),
                            "to get framework objects".to_string(),
                        ))
                    })?;
                framework_objects
            }
        };

        get_all_version_syncs(&self.project, !unimplemented!()).map_err(|err| {
            RoutineFailure::error(Message::new(
                "Failed".to_string(),
                "to generate migrations".to_string(),
            ))
        })?;

        Ok(RoutineSuccess::success(Message {
            action: "Generated".to_string(),
            details: "migrations".to_string(),
        }))

        // generate_version_syncs(
        //     &self.project,
        //     &self.project.framework_object_versions(),
        //     &self.project.version_sync_list(),
        // );
        //
        // Ok(RoutineSuccess::success(Message {
        //     action: "Generated".to_string(),
        //     details: "migrations".to_string(),
        // }))
    }
}
