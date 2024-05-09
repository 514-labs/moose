use log::{error, info};
use std::{fs, io::Write, process::Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::cli::display::{Message, MessageType};
use crate::framework::controller::FrameworkObjectVersions;
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

    show_message!(
        MessageType::Success,
        Message {
            action: "Created".to_string(),
            details: "flow".to_string(),
        }
    );

    let mut missing_datamodels = Vec::new();
    if verify_datamodel(project, source.clone()).is_err() {
        missing_datamodels.push(source);
    }
    if verify_datamodel(project, destination.clone()).is_err() {
        missing_datamodels.push(destination);
    }
    if !missing_datamodels.is_empty() {
        {
            show_message!(
                MessageType::Info,
                Message {
                    action: "".to_string(),
                    details: "\n".to_string(),
                }
            );
        }

        let missing_datamodels_str = missing_datamodels.join(", ");
        show_message!(
            MessageType::Highlight,
            Message {
                action: "Next steps".to_string(),
                details: format!(
                    "You may be missing the following datamodels. Add these to {}: {}",
                    project.schemas_dir().display(),
                    missing_datamodels_str
                )
            }
        );
    }

    Ok(RoutineSuccess::success(Message::new(
        "Created".to_string(),
        format!("flow {}", flow_file_path.display()),
    )))
}

pub fn verify_datamodel(project: &Project, datamodel: String) -> anyhow::Result<()> {
    let child = std::process::Command::new("grep")
        .arg("-r")
        .arg("--include=*.ts")
        .arg("-E")
        .arg(format!("export\\s+interface\\s+\\b{}\\b", &datamodel))
        .arg(project.schemas_dir())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Failed to search datamodels for {}",
            &datamodel
        ))
    }
}

pub fn verify_flows_against_datamodels(
    project: &Project,
    framework_object_versions: &FrameworkObjectVersions,
) -> anyhow::Result<()> {
    let flows = project.get_flows();
    let flows_dir = project.flows_dir();

    let mut flows_with_missing_models = Vec::<String>::new();
    for (source, destinations) in flows {
        if !framework_object_versions
            .current_models
            .models
            .contains_key(&source)
        {
            flows_with_missing_models.push(format!("{}/{}", flows_dir.display(), source));
        }

        destinations.iter().for_each(|destination| {
            if !framework_object_versions
                .current_models
                .models
                .contains_key(destination)
            {
                flows_with_missing_models.push(format!(
                    "{}/{}/{}",
                    flows_dir.display(),
                    source,
                    destination
                ));
            }
        });
    }

    if !flows_with_missing_models.is_empty() {
        flows_with_missing_models.sort();
        show_message!(
            MessageType::Error,
            Message {
                action: "Flow".to_string(),
                details: "These flow sources/destinations have missing data models. Add the data models or rename the flow directories".to_string(),
            }
        );
        flows_with_missing_models.iter().for_each(|flow_path| {
            show_message!(
                MessageType::Error,
                Message {
                    action: "".to_string(),
                    details: flow_path.to_string(),
                }
            );
        });
    }

    Ok(())
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
