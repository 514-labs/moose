use std::{fs, io::Write};

use crate::{
    cli::display::Message, framework::typescript::templates::BASE_AGGREGATION_TEMPLATE,
    project::Project,
};

use super::{RoutineFailure, RoutineSuccess};

pub fn create_aggregation_file(
    project: &Project,
    filename: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    let aggregations_dir = project.aggregations_dir();
    let aggregation_file_path = aggregations_dir.join(format!("{}.ts", filename));

    let mut aggregation_file = fs::File::create(&aggregation_file_path).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!(
                    "to create aggregation file {}",
                    aggregation_file_path.display()
                ),
            ),
            err,
        )
    })?;

    aggregation_file
        .write_all(BASE_AGGREGATION_TEMPLATE.as_bytes())
        .map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to write to aggregation file {}",
                        aggregation_file_path.display()
                    ),
                ),
                err,
            )
        })?;

    Ok(RoutineSuccess::success(Message::new(
        "Created".to_string(),
        format!("aggregation {}", aggregation_file_path.display()),
    )))
}
