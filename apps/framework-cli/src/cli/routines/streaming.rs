use super::{RoutineFailure, RoutineSuccess};
use crate::cli::display::{Message, MessageType};
use crate::framework::core::code_loader::{
    load_framework_objects, FrameworkObject, FrameworkObjectVersions,
};
use crate::framework::data_model::model::DataModel;
use crate::framework::streaming;
use crate::project::Project;
use itertools::Either;
use log::debug;
use std::collections::HashMap;
use std::{fs, io::Write, process::Stdio};

pub async fn create_streaming_function_file(
    project: &Project,
    source: String,
    destination: String,
) -> Result<RoutineSuccess, RoutineFailure> {
    let framework_objects = load_framework_objects(project).await;
    let empty_map = HashMap::new();
    let (models, error_occurred) = match &framework_objects {
        Ok(framework_objects) => (&framework_objects.current_models.models, false),
        Err(err) => {
            debug!(
                "Failed to crawl schema while creating streaming function: {:?}",
                err
            );
            (&empty_map, true)
        }
    };

    fn try_get_from_models<'a>(
        name: &'a str,
        models: &'a HashMap<String, FrameworkObject>,
    ) -> Either<&'a DataModel, &'a str> {
        match models.get(name) {
            None => Either::Right(name),
            Some(framework_object) => Either::Left(&framework_object.data_model),
        }
    }

    let function_file_content = streaming::generate::generate(
        project,
        try_get_from_models(&source, models),
        try_get_from_models(&destination, models),
    );

    let function_file_path = project.streaming_func_dir().join(format!(
        "{}__{}.{}",
        source,
        destination,
        project.language.extension()
    ));

    let mut function_file = fs::File::create(function_file_path.as_path()).map_err(|err| {
        RoutineFailure::new(
            Message::new(
                "Failed".to_string(),
                format!(
                    "to create streaming function file in {}",
                    function_file_path.display()
                ),
            ),
            err,
        )
    })?;
    function_file
        .write_all(function_file_content.as_bytes())
        .map_err(|err| {
            RoutineFailure::new(
                Message::new(
                    "Failed".to_string(),
                    format!(
                        "to write to streaming function file in {}",
                        function_file_path.display()
                    ),
                ),
                err,
            )
        })?;

    if error_occurred || !models.contains_key(&source) || !models.contains_key(&destination) {
        verify_datamodels_with_grep(project, &source, &destination);
    } else {
        verify_datamodels_with_framework_objects(models, &source, &destination);
    }

    Ok(RoutineSuccess {
        message_type: MessageType::Success,
        message: Message {
            action: "Created".to_string(),
            details: "streaming function".to_string(),
        },
    })
}

pub fn verify_streaming_functions_against_datamodels(
    project: &Project,
    framework_object_versions: &FrameworkObjectVersions,
) -> anyhow::Result<()> {
    let functions = project.get_functions();
    let functions_dir = project.streaming_func_dir();

    let mut functions_with_missing_models = Vec::<String>::new();
    for (source, destinations) in functions {
        if !framework_object_versions
            .current_models
            .models
            .contains_key(&source)
        {
            functions_with_missing_models.push(format!("{}/{}", functions_dir.display(), source));
        }

        destinations.iter().for_each(|destination| {
            if !framework_object_versions
                .current_models
                .models
                .contains_key(destination)
            {
                functions_with_missing_models.push(format!(
                    "{}/{}/{}",
                    functions_dir.display(),
                    source,
                    destination
                ));
            }
        });
    }

    if !functions_with_missing_models.is_empty() {
        functions_with_missing_models.sort();
        show_message!(
            MessageType::Error,
            Message {
                action: "Function".to_string(),
                details: "These functions sources/destinations have missing data models. Add the data models or rename the streaming functions".to_string(),
            }
        );
        functions_with_missing_models
            .iter()
            .for_each(|function_path| {
                show_message!(
                    MessageType::Error,
                    Message {
                        action: "".to_string(),
                        details: function_path.to_string(),
                    }
                );
            });
    }

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
        show_missing_datamodels_messages(&missing_datamodels);
    }
}

fn grep_datamodel(project: &Project, datamodel: &str) -> anyhow::Result<()> {
    let child = std::process::Command::new("grep")
        .arg("-r")
        .arg("--include=*.ts")
        .arg("-E")
        .arg(format!("export\\s+interface\\s+\\b{}\\b", &datamodel))
        .arg(project.data_models_dir())
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
        show_missing_datamodels_messages(&missing_datamodels);
    }
}

fn show_missing_datamodels_messages(missing_datamodels: &[&str]) {
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
