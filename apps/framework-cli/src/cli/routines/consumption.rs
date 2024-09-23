use std::{fs, io::Write};

use crate::{
    cli::display::Message,
    framework::{
        python::templates::PYTHON_BASE_CONSUMPTION_TEMPLATE,
        typescript::templates::TS_BASE_CONSUMPTION_TEMPLATE,
    },
    project::Project,
};

use crate::framework::languages::SupportedLanguages;

use super::{RoutineFailure, RoutineSuccess};

pub fn create_consumption_file(
    project: &Project,
    filename: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    let apis_dir = project.consumption_dir();
    let apis_file_path = apis_dir.join(format!("{}.{}", filename, project.language.extension()));
    let template = match project.language {
        SupportedLanguages::Typescript => TS_BASE_CONSUMPTION_TEMPLATE,
        SupportedLanguages::Python => PYTHON_BASE_CONSUMPTION_TEMPLATE,
    };

    let mut apis_file = fs::File::create(&apis_file_path).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!("to create consumption file {}", apis_file_path.display()),
            ),
            err,
        )
    })?;

    apis_file.write_all(template.as_bytes()).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!("to write to consumption file {}", apis_file_path.display()),
            ),
            err,
        )
    })?;

    Ok(RoutineSuccess::success(Message::new(
        "Created".to_string(),
        format!("consumption api {}", apis_file_path.display()),
    )))
}
