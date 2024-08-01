use std::{fs, io::Write};

use crate::{
    cli::display::Message, framework::typescript::templates::BASE_BLOCK_TEMPLATE, project::Project,
};

use super::{RoutineFailure, RoutineSuccess};

pub async fn create_block_file(
    project: &Project,
    filename: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    let blocks_dir = project.blocks_dir();
    let block_file_path = blocks_dir.join(format!("{}.ts", filename));

    let mut block_file = fs::File::create(&block_file_path).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!("to create block file {}", block_file_path.display()),
            ),
            err,
        )
    })?;

    block_file
        .write_all(BASE_BLOCK_TEMPLATE.as_bytes())
        .map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!("to write to block file {}", block_file_path.display()),
                ),
                err,
            )
        })?;

    Ok(RoutineSuccess::success(Message::new(
        "Created".to_string(),
        "block".to_string(),
    )))
}
