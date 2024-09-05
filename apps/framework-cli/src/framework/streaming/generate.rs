use crate::framework::core::code_loader::FrameworkObject;
use crate::framework::core::infrastructure::table::ColumnType;
use crate::framework::data_model::model::DataModel;
use crate::framework::languages::SupportedLanguages;
use crate::framework::python;
use crate::framework::python::templates::PYTHON_BASE_STREAMING_FUNCTION_TEMPLATE;
use crate::framework::typescript::templates::TS_BASE_STREAMING_FUNCTION_TEMPLATE;
use crate::project::Project;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn import_line(lang: SupportedLanguages, path: &str, names: &[&str]) -> String {
    match lang {
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

pub fn generate(project: &Project, source_dm: &DataModel, destination_dm: &DataModel) -> String {
    let template = match project.language {
        SupportedLanguages::Typescript => TS_BASE_STREAMING_FUNCTION_TEMPLATE,
        SupportedLanguages::Python => PYTHON_BASE_STREAMING_FUNCTION_TEMPLATE,
    };

    let is_migration = source_dm.version != destination_dm.version;

    let mut maybe_old_name = String::default();
    let (source_type, destination_type) = if source_dm.name == destination_dm.name {
        assert_ne!(source_dm.version, destination_dm.version);

        maybe_old_name.push_str(&source_dm.name);
        maybe_old_name.push_str("Old");
        (&source_dm.name, &maybe_old_name)
    } else {
        (&source_dm.name, &destination_dm.name)
    };

    let mut function_file_template = template
        .to_string()
        .replace("{{source}}", source_type)
        .replace("{{destination}}", destination_type);

    // let function_file_path = project.streaming_func_dir().join(format!(
    //     "{}__{}.{}",
    //     source_type,
    //     destination_type,
    //     project.language.extension()
    // ));

    let source_path = get_import_path(source_dm, project);
    let destination_path = get_import_path(destination_dm, project);

    let (source_import, destination_import) = if source_path == destination_path {
        (
            import_line(
                project.language,
                &source_path,
                &[source_type, destination_type],
            ),
            "".to_string(),
        )
    } else {
        (
            import_line(project.language, &source_path, &[source_type]),
            import_line(project.language, &destination_path, &[destination_type]),
        )
    };

    function_file_template = function_file_template
        .replace("{{source_import}}", &source_import)
        .replace("{{destination_import}}", &destination_import);

    if let Some(dest_model) = Some(destination_dm) {
        let mut destination_object = "".to_string();
        match project.language {
            SupportedLanguages::Typescript if is_migration => {
                // let the compiler figure out the rest
                destination_object.push_str("{ ...source }")
            }
            SupportedLanguages::Typescript => {
                destination_object.push_str("{\n");
                dest_model.columns.iter().for_each(|field| {
                    destination_object.push_str(&format!(
                        "    {}: {},\n",
                        field.name,
                        get_default_value_for_type(&field.data_type, project.language)
                    ));
                });
                destination_object.push_str("  }");
            }
            SupportedLanguages::Python => {
                destination_object.push_str(destination_type);
                destination_object.push_str("(\n");
                dest_model.columns.iter().for_each(|field| {
                    destination_object.push_str(&format!(
                        "        {}={},\n",
                        field.name,
                        get_default_value_for_type(&field.data_type, project.language)
                    ));
                });
                destination_object.push_str("    )");
            }
        }

        function_file_template =
            function_file_template.replace("{{destination_object}}", &destination_object);
    } else {
        let empty = match project.language {
            SupportedLanguages::Typescript => "null",
            SupportedLanguages::Python => "None",
        };
        function_file_template = function_file_template.replace("{{destination_object}}", empty);
    }

    function_file_template
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

fn get_import_path(data_model: &DataModel, project: &Project) -> String {
    match data_model
        .abs_file_path
        .strip_prefix(project.old_version_location(&data_model.version).unwrap())
    {
        Ok(relative_path) => match project.language {
            SupportedLanguages::Typescript => {
                format!(
                    "versions/{}/{}",
                    &data_model.version,
                    relative_path.with_extension("").to_string_lossy()
                )
            }
            SupportedLanguages::Python => python_path_to_module(
                relative_path,
                Some(python::version_to_identifier(&data_model.version)),
            ),
        },
        Err(_) => {
            assert_eq!(data_model.version, project.cur_version());
            match project.language {
                SupportedLanguages::Typescript => format!(
                    "datamodels/{}",
                    data_model
                        .abs_file_path
                        .with_extension("")
                        .strip_prefix(project.data_models_dir())
                        .unwrap()
                        .to_string_lossy()
                ),
                SupportedLanguages::Python => {
                    let relative_path_from_root = data_model
                        .abs_file_path
                        .strip_prefix(&project.project_location)
                        .unwrap_or(&data_model.abs_file_path);

                    python_path_to_module(relative_path_from_root, None)
                }
            }
        }
    }
}

fn python_path_to_module(relative_file_path: &Path, base: Option<String>) -> String {
    let relative_file_path = relative_file_path.with_extension("");

    let mut path = base.unwrap_or("".to_string());
    for path_segment in &relative_file_path {
        if !path.is_empty() {
            path.push('.');
        }
        path.push_str(&path_segment.to_string_lossy())
    }
    path
}
