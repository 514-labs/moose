use crate::utilities::constants::{SAMPLE_FLOWS_DEST, SAMPLE_FLOWS_SOURCE};
use crate::{cli::display::Message, project::Project};

use super::flow::create_flow_directory;
use super::{RoutineFailure, RoutineSuccess};

pub fn initialize_project(project: &Project) -> Result<RoutineSuccess, RoutineFailure> {
    project.setup_app_dir().map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to create 'app' directory. Check permissions or contact us`".to_string(),
            ),
            err,
        )
    })?;

    create_flow_directory(
        project,
        SAMPLE_FLOWS_SOURCE.to_string(),
        SAMPLE_FLOWS_DEST.to_string(),
    )?
    .show();

    project.create_base_app_files().map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to create 'app' files, Check permissions or contact us".to_string(),
            ),
            err,
        )
    })?;

    project.create_vscode_files().map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                "to create 'vscode' files, Check permissions or contact us".to_string(),
            ),
            err,
        )
    })?;

    Ok(RoutineSuccess::info(Message::new(
        "Created".to_string(),
        "starting `app` files".to_string(),
    )))
}
