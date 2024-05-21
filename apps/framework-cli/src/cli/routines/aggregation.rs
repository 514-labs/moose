use log::{error, info};
use std::process::Stdio;
use std::{fs, io::Write};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::{
    cli::display::Message,
    framework::typescript::templates::BASE_AGGREGATION_TEMPLATE,
    project::Project,
    utilities::constants::{DENO_AGGREGATIONS, DENO_DIR},
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

pub fn start_aggregation_process(project: &Project) -> anyhow::Result<()> {
    let project_root_path = project.project_location.clone();
    let deno_file = project
        .internal_dir()
        .unwrap()
        .join(DENO_DIR)
        .join(DENO_AGGREGATIONS);

    let mut child = Command::new("deno")
        .current_dir(&project_root_path)
        .arg("run")
        .arg("--allow-all")
        .arg(deno_file)
        .arg(&project_root_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start aggregation");

    let stdout = child
        .stdout
        .take()
        .expect("Aggregation process did not have a handle to stdout");

    let stderr = child
        .stderr
        .take()
        .expect("Aggregation process did not have a handle to stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            info!("{}", line);
        }
    });

    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            error!("{}", line);
        }
    });

    Ok(())
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
