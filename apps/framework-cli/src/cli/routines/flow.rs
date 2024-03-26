use std::{fs, io::Write, sync::Arc};

use crate::{
    cli::display::Message, framework::schema::templates::BASE_FLOW_TEMPLATE, project::Project,
    utilities::constants::FLOW_FILE,
};

use super::{Routine, RoutineFailure, RoutineSuccess};

pub struct CreateFlowDirectory {
    project: Arc<Project>,
    source: String,
    destination: String,
}

impl CreateFlowDirectory {
    pub fn new(project: Arc<Project>, source: String, destination: String) -> Self {
        Self {
            project,
            source,
            destination,
        }
    }
}

impl Routine for CreateFlowDirectory {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let flows_dir = self.project.flows_dir();
        let source_destination_dir = flows_dir
            .join(self.source.clone())
            .join(self.destination.clone());

        match fs::create_dir_all(source_destination_dir.clone()) {
            Ok(_) => Ok(RoutineSuccess::success(Message::new(
                "Created".to_string(),
                "flow directory".to_string(),
            ))),
            Err(err) => Err(RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to create flow directory {}",
                        source_destination_dir.display()
                    ),
                ),
                err,
            )),
        }
    }
}

pub struct CreateFlowFile {
    project: Arc<Project>,
    source: String,
    destination: String,
}

impl CreateFlowFile {
    pub fn new(project: Arc<Project>, source: String, destination: String) -> Self {
        Self {
            project,
            source,
            destination,
        }
    }
}

impl Routine for CreateFlowFile {
    fn run_silent(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let flows_dir = self.project.flows_dir();
        let flow_file_path = flows_dir
            .join(&self.source)
            .join(&self.destination)
            .join(FLOW_FILE);

        let mut flow_file = fs::File::create(&flow_file_path).map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!("to create flow file in {}", flow_file_path.display()),
                ),
                err,
            )
        })?;

        flow_file
            .write_all(
                BASE_FLOW_TEMPLATE
                    .to_string()
                    .replace("{{project_name}}", &self.project.name())
                    .replace("{{source}}", &self.source)
                    .replace("{{destination}}", &self.destination)
                    .as_bytes(),
            )
            .map_err(|err| {
                RoutineFailure::new(
                    Message::new(
                        "Failed".to_string(),
                        format!("to write to flow file in {}", flow_file_path.display()),
                    ),
                    err,
                )
            })?;

        Ok(RoutineSuccess::success(Message::new(
            "Successfully".to_string(),
            format!("created flow {}", flow_file_path.display()),
        )))
    }
}
