use log::{debug, error, info};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io::Write, process::Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::cli::display::{Message, MessageType};
use crate::framework::controller::{FrameworkObject, FrameworkObjectVersions};
use crate::framework::data_model::schema::ColumnType;
use crate::framework::typescript::templates::BASE_FLOW_TEMPLATE;
use crate::project::Project;
use crate::utilities::constants::{DENO_DIR, DENO_TRANSFORM, FLOW_FILE};

use super::{crawl_schema, RoutineFailure, RoutineSuccess};

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

    let old_versions = project.old_versions_sorted();
    match crawl_schema(project, &old_versions) {
        Ok(framework_objects) => {
            if !framework_objects
                .current_models
                .models
                .contains_key(&destination)
            {
                write_null_to_flow_file(project, flow_file_path, &source, &destination)?;
                verify_datamodels_with_grep(project, &source, &destination);
            } else {
                write_destination_object_to_flow_file(
                    project,
                    flow_file_path,
                    &framework_objects.current_models.models,
                    &source,
                    &destination,
                )?;
                verify_datamodels_with_framework_objects(
                    project,
                    &framework_objects.current_models.models,
                    &source,
                    &destination,
                );
            }
        }
        Err(err) => {
            debug!("Failed to crawl schema while creating flow: {:?}", err);
            write_null_to_flow_file(project, flow_file_path, &source, &destination)?;
            verify_datamodels_with_grep(project, &source, &destination);
        }
    }

    Ok(RoutineSuccess::success(Message::new(
        "Created".to_string(),
        "flow".to_string(),
    )))
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

fn verify_datamodels_with_grep(project: &Project, source: &str, destination: &str) {
    let mut missing_datamodels = Vec::new();

    if grep_datamodel(project, source).is_err() {
        missing_datamodels.push(source);
    }
    if grep_datamodel(project, destination).is_err() {
        missing_datamodels.push(destination);
    }

    if !missing_datamodels.is_empty() {
        show_missing_datamodels_messages(&missing_datamodels, project);
    }
}

fn grep_datamodel(project: &Project, datamodel: &str) -> anyhow::Result<()> {
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

fn verify_datamodels_with_framework_objects(
    project: &Project,
    models: &HashMap<String, FrameworkObject>,
    source: &str,
    destination: &str,
) {
    let mut missing_datamodels = Vec::new();

    if !models.contains_key(source) {
        missing_datamodels.push(source);
    }
    if !models.contains_key(destination) {
        missing_datamodels.push(destination);
    }

    if !missing_datamodels.is_empty() {
        show_missing_datamodels_messages(&missing_datamodels, project);
    }
}

fn show_missing_datamodels_messages(missing_datamodels: &[&str], project: &Project) {
    {
        show_message!(
            MessageType::Info,
            Message {
                action: "".to_string(),
                details: "\n".to_string(),
            }
        );
    }

    let missing_datamodels_str = missing_datamodels.join("\n\t- ");
    show_message!(
        MessageType::Highlight,
        Message {
            action: "Next steps".to_string(),
            details: format!(
                "\n\n📂 You may be missing the following datamodels. Add these to your datamodels directory:\n\t- {}",
                missing_datamodels_str
            )
        }
    );
}

fn write_null_to_flow_file(
    project: &Project,
    flow_file_path: PathBuf,
    source: &str,
    destination: &str,
) -> Result<(), RoutineFailure> {
    write_to_flow_file(project, flow_file_path, source, destination, "null")
}

fn write_destination_object_to_flow_file(
    project: &Project,
    flow_file_path: PathBuf,
    models: &HashMap<String, FrameworkObject>,
    source: &str,
    destination: &str,
) -> Result<(), RoutineFailure> {
    let mut destination_object = "{\n".to_string();
    models
        .get(destination)
        .unwrap()
        .data_model
        .columns
        .iter()
        .for_each(|field| {
            destination_object.push_str(&format!(
                "    {}: {},\n",
                field.name,
                get_default_value_for_type(&field.data_type)
            ));
        });
    destination_object.push_str("  }");

    write_to_flow_file(
        project,
        flow_file_path,
        source,
        destination,
        &destination_object,
    )
}

fn write_to_flow_file(
    project: &Project,
    flow_file_path: PathBuf,
    source: &str,
    destination: &str,
    destination_object: &str,
) -> Result<(), RoutineFailure> {
    let mut flow_file = fs::File::create(&flow_file_path).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!("to create flow file in {}", flow_file_path.display()),
            ),
            err,
        )
    })?;

    let _ = flow_file
        .write_all(
            BASE_FLOW_TEMPLATE
                .to_string()
                .replace("{{project_name}}", &project.name())
                .replace("{{source}}", source)
                .replace("{{destination}}", destination)
                .replace("{{destination_object}}", destination_object)
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
        });

    show_message!(
        MessageType::Success,
        Message {
            action: "Created".to_string(),
            details: "flow".to_string(),
        }
    );

    Ok(())
}

fn get_default_value_for_type(column_type: &ColumnType) -> String {
    match column_type {
        ColumnType::String => "\"\"".to_string(),
        ColumnType::Boolean => "false".to_string(),
        ColumnType::Int => "0".to_string(),
        ColumnType::BigInt => "0".to_string(),
        ColumnType::Float => "0".to_string(),
        ColumnType::Decimal => "0".to_string(),
        ColumnType::DateTime => "new Date()".to_string(),
        ColumnType::Enum(_) => "any".to_string(),
        ColumnType::Array(_) => "[]".to_string(),
        ColumnType::Json => "{}".to_string(),
        ColumnType::Bytes => "[]".to_string(),
    }
}
