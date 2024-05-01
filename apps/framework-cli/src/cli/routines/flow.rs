use log::{error, info};
use std::{fs, io::Write, process::Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::cli::display::Message;
use crate::framework::typescript::templates::BASE_FLOW_TEMPLATE;
use crate::project::Project;
use crate::utilities::constants::{DENO_DIR, DENO_TRANSFORM, FLOW_FILE};

use super::{RoutineFailure, RoutineSuccess};

pub fn create_flow_directory(
    project: &Project,
    source: String,
    destination: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    let flows_dir = project.flows_dir();
    let source_destination_dir = flows_dir.join(source.clone()).join(destination.clone());

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

pub fn create_flow_file(
    project: &Project,
    source: String,
    destination: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    let flows_dir = project.flows_dir();
    let flow_file_path = flows_dir
        .join(source.clone())
        .join(destination.clone())
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
                .replace("{{project_name}}", &project.name())
                .replace("{{source}}", &source)
                .replace("{{destination}}", &destination)
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
        "Created".to_string(),
        format!("flow {}", flow_file_path.display()),
    )))
}

pub fn start_flow_process(project: &Project) -> anyhow::Result<()> {
    let project_root_path = project.project_location.clone();
    let deno_file = project
        .internal_dir()
        .unwrap()
        .join(DENO_DIR)
        .join(DENO_TRANSFORM);

    let mut child = Command::new("deno")
        .current_dir(&project_root_path)
        .arg("run")
        .arg("--allow-all")
        .arg(deno_file)
        .arg(&project_root_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start deno");

    let stdout = child
        .stdout
        .take()
        .expect("Deno process did not have a handle to stdout");

    let stderr = child
        .stderr
        .take()
        .expect("Deno process did not have a handle to stderr");

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
