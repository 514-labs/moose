use std::{fs, io::Write, path::PathBuf};

use super::{RoutineFailure, RoutineSuccess};
use crate::{
    cli::display::Message,
    framework::{
        languages::SupportedLanguages, python::templates::PYTHON_BASE_BLOCKS_TEMPLATE,
        typescript::templates::TS_BASE_BLOCK_TEMPLATE,
    },
    project::Project,
};

pub async fn create_block_file(
    project: &Project,
    filename: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    let blocks_dir = project.blocks_dir();
    let (extension, template) = match project.language {
        SupportedLanguages::Typescript => ("ts", TS_BASE_BLOCK_TEMPLATE),
        SupportedLanguages::Python => ("py", PYTHON_BASE_BLOCKS_TEMPLATE),
    };

    let block_file_path = blocks_dir.join(format!("{}.{}", filename, extension));
    create_and_write_file(&block_file_path, template)?;

    Ok(RoutineSuccess::success(Message::new(
        "Created".to_string(),
        "block".to_string(),
    )))
}

fn create_and_write_file(path: &PathBuf, content: &str) -> Result<(), RoutineFailure> {
    let mut block_file = fs::File::create(path).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!("to create block file {}", path.display()),
            ),
            err,
        )
    })?;

    block_file.write_all(content.as_bytes()).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!("to write to block file {}", path.display()),
            ),
            err,
        )
    })?;

    Ok(())
}
