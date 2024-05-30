use std::{fs, io::Write};

use crate::{
    cli::display::Message, framework::typescript::templates::BASE_AGGREGATION_TEMPLATE,
    project::Project,
};

use super::{crawl_schema, RoutineFailure, RoutineSuccess};

pub async fn create_aggregation_file(
    project: &Project,
    filename: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    let old_versions = project.old_versions_sorted();
    let framework_objects = crawl_schema(project, &old_versions).await.map_err(|err| {
        RoutineFailure::error(Message::new(
            "Failed".to_string(),
            format!("to create aggregation: {}", err),
        ))
    })?;

    if framework_objects
        .current_models
        .models
        .contains_key(&filename)
    {
        return Err(RoutineFailure::error(Message::new(
            "Failed".to_string(),
            format!("Model & aggregation {} cannot have the same name", filename),
        )));
    }

    write_aggregation_file(project, filename)
}

fn write_aggregation_file(
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
        "aggregation".to_string(),
    )))
}
