use crate::cli::display::{Message, MessageType};
use crate::framework::core::code_loader::{
    load_framework_objects, FrameworkObject, FrameworkObjectVersions,
};
use crate::framework::core::infrastructure::table::ColumnType;
use crate::framework::languages::SupportedLanguages;
use crate::framework::python::templates::PYTHON_BASE_STREAMING_FUNCTION_TEMPLATE;
use crate::framework::typescript::templates::TS_BASE_STREAMING_FUNCTION_TEMPLATE;
use crate::project::Project;
use log::debug;
use pathdiff::diff_paths;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, io::Write, process::Stdio};

use super::{RoutineFailure, RoutineSuccess};

pub struct StreamingFunctionFileBuilder {
    language: SupportedLanguages,
    function_file_path: PathBuf,
    function_file_template: String,
}

impl StreamingFunctionFileBuilder {
    pub fn new(project: &Project, source: &str, destination: &str) -> Self {
        let language = project.language;
        let template = match language {
            SupportedLanguages::Typescript => TS_BASE_STREAMING_FUNCTION_TEMPLATE,
            SupportedLanguages::Python => PYTHON_BASE_STREAMING_FUNCTION_TEMPLATE,
        };

        let function_file_template = template
            .to_string()
            .replace("{{source}}", source)
            .replace("{{destination}}", destination);

        let function_file_path = project.streaming_func_dir().join(format!(
            "{}__{}.{}",
            source,
            destination,
            language.extension()
        ));

        Self {
            language,
            function_file_path,
            function_file_template,
        }
    }

    pub fn return_object(
        &mut self,
        destination: &str,
        models: &HashMap<String, FrameworkObject>,
    ) -> &mut Self {
        if let Some(dest_model) = models.get(destination) {
            let mut destination_object = "".to_string();
            match self.language {
                SupportedLanguages::Typescript => {
                    destination_object.push_str("{\n");
                    dest_model.data_model.columns.iter().for_each(|field| {
                        destination_object.push_str(&format!(
                            "    {}: {},\n",
                            field.name,
                            get_default_value_for_type(&field.data_type, self.language)
                        ));
                    });
                    destination_object.push_str("  }");
                }
                SupportedLanguages::Python => {
                    destination_object.push_str(destination);
                    destination_object.push_str("(\n");
                    dest_model.data_model.columns.iter().for_each(|field| {
                        destination_object.push_str(&format!(
                            "        {}={},\n",
                            field.name,
                            get_default_value_for_type(&field.data_type, self.language)
                        ));
                    });
                    destination_object.push_str("    )");
                }
            }

            self.function_file_template = self
                .function_file_template
                .replace("{{destination_object}}", &destination_object);
        } else {
            let empty = match self.language {
                SupportedLanguages::Typescript => "null",
                SupportedLanguages::Python => "None",
            };
            self.function_file_template = self
                .function_file_template
                .replace("{{destination_object}}", empty);
        }

        self
    }

    fn import_line(&self, path: &str, names: &[&str]) -> String {
        match self.language {
            SupportedLanguages::Typescript => {
                let names = names.join(", ");
                format!("import {{ {} }} from \"{}\";", names, path)
            }
            SupportedLanguages::Python => {
                let names = names.join(", ");
                format!("from {} import {}", path, names)
            }
        }
    }

    pub fn imports(
        &mut self,
        source: &str,
        destination: &str,
        models: &HashMap<String, FrameworkObject>,
        project: &Project,
    ) -> &mut Self {
        let source_path = self.get_model_path(source, models, project);
        let destination_path = self.get_model_path(destination, models, project);

        let (source_import, destination_import) = if source_path == destination_path {
            (
                self.import_line(&source_path, &[source, destination]),
                "".to_string(),
            )
        } else {
            (
                self.import_line(&source_path, &[source]),
                self.import_line(&destination_path, &[destination]),
            )
        };

        self.function_file_template = self
            .function_file_template
            .replace("{{source_import}}", &source_import)
            .replace("{{destination_import}}", &destination_import);

        self
    }

    pub fn get_model_path(
        &self,
        target_model: &str,
        models: &HashMap<String, FrameworkObject>,
        project: &Project,
    ) -> String {
        if let Some(model) = models.get(target_model) {
            let model_path = model.original_file_path.clone();

            match self.language {
                SupportedLanguages::Typescript => {
                    let mut model_relative_path = diff_paths(
                        model_path.clone(),
                        self.function_file_path.parent().unwrap(),
                    )
                    .unwrap_or(model_path);
                    model_relative_path.set_extension("");

                    model_relative_path.to_string_lossy().to_string()
                }
                SupportedLanguages::Python => {
                    let relative_path_from_root = model_path
                        .strip_prefix(&project.project_location)
                        .unwrap_or(&model_path)
                        .with_extension("");
                    let mut path = "".to_string();
                    for path_segment in &relative_path_from_root {
                        if !path.is_empty() {
                            path.push('.');
                        }
                        path.push_str(&path_segment.to_string_lossy())
                    }
                    path
                }
            }
        } else {
            match self.language {
                SupportedLanguages::Typescript => "../datamodels/models".to_string(),
                SupportedLanguages::Python => "app.datamodels.models".to_string(),
            }
        }
    }

    pub fn write(&self) -> Result<RoutineSuccess, RoutineFailure> {
        let mut function_file =
            fs::File::create(self.function_file_path.as_path()).map_err(|err| {
                RoutineFailure::new(
                    Message::new(
                        "Failed".to_string(),
                        format!(
                            "to create streaming function file in {}",
                            self.function_file_path.display()
                        ),
                    ),
                    err,
                )
            })?;

        let _ = function_file
            .write_all(self.function_file_template.as_bytes())
            .map_err(|err| {
                RoutineFailure::new(
                    Message::new(
                        "Failed".to_string(),
                        format!(
                            "to write to streaming function file in {}",
                            self.function_file_path.display()
                        ),
                    ),
                    err,
                )
            });

        Ok(RoutineSuccess {
            message_type: MessageType::Success,
            message: Message {
                action: "Created".to_string(),
                details: "streaming function".to_string(),
            },
        })
    }
}

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

    let success = StreamingFunctionFileBuilder::new(project, &source, &destination)
        .imports(&source, &destination, models, project)
        .return_object(&destination, models)
        .write()?;

    if error_occurred || !models.contains_key(&source) || !models.contains_key(&destination) {
        verify_datamodels_with_grep(project, &source, &destination);
    } else {
        verify_datamodels_with_framework_objects(models, &source, &destination);
    }

    Ok(success)
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

fn get_default_value_for_type(column_type: &ColumnType, lang: SupportedLanguages) -> String {
    match (column_type, lang) {
        (ColumnType::String, _) => "\"\"".to_string(),
        (ColumnType::Boolean, _) => "false".to_string(),
        (ColumnType::Int, _) => "0".to_string(),
        (ColumnType::BigInt, _) => "0".to_string(),
        (ColumnType::Float, SupportedLanguages::Typescript) => "0".to_string(),
        (ColumnType::Float, SupportedLanguages::Python) => "0.0".to_string(),
        (ColumnType::Decimal, _) => "0".to_string(),
        (ColumnType::DateTime, SupportedLanguages::Typescript) => "new Date()".to_string(),
        (ColumnType::DateTime, SupportedLanguages::Python) => "datetime.now()".to_string(),
        (ColumnType::Enum(_), _) => "any".to_string(),
        (ColumnType::Array(_), _) => "[]".to_string(),
        (ColumnType::Nested(_), SupportedLanguages::Typescript) => "{}".to_string(),
        (ColumnType::Nested(inner), SupportedLanguages::Python) => format!("{}()", inner.name),
        (ColumnType::Json, _) => "{}".to_string(),
        (ColumnType::Bytes, _) => "[]".to_string(),
    }
}
