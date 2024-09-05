use std::path::Path;

use itertools::Either;

use crate::framework::core::infrastructure::table::ColumnType;
use crate::framework::data_model::model::DataModel;
use crate::framework::languages::SupportedLanguages;
use crate::framework::python;
use crate::framework::python::templates::PYTHON_BASE_STREAMING_FUNCTION_TEMPLATE;
use crate::framework::typescript::templates::TS_BASE_STREAMING_FUNCTION_TEMPLATE;
use crate::project::Project;

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

pub fn generate(
    project: &Project,
    source_dm: Either<&DataModel, &str>,
    destination_dm: Either<&DataModel, &str>,
) -> String {
    let template = match project.language {
        SupportedLanguages::Typescript => TS_BASE_STREAMING_FUNCTION_TEMPLATE,
        SupportedLanguages::Python => PYTHON_BASE_STREAMING_FUNCTION_TEMPLATE,
    };

    let is_migration = source_dm.left().is_some_and(|source| {
        destination_dm
            .left()
            .is_some_and(|destination| source.version != destination.version)
    });

    let mut maybe_old_name = String::default();
    let (source_type, destination_type): (&str, &str) = match (source_dm, destination_dm) {
        (Either::Left(source), Either::Left(dest)) if is_migration => {
            if source.name == dest.name {
                maybe_old_name.push_str(&source.name);
                maybe_old_name.push_str("Old");
                (&maybe_old_name, &dest.name)
            } else {
                (&source.name, &dest.name)
            }
        }
        (Either::Left(source), Either::Left(dest)) => (&source.name, &dest.name),
        (Either::Left(source), Either::Right(dest)) => (&source.name, dest),
        (Either::Right(source), Either::Left(dest)) => (source, &dest.name),
        (Either::Right(source), Either::Right(dest)) => (source, dest),
    };
    assert_ne!(source_type, destination_type);

    let mut function_file_template = template
        .to_string()
        .replace("{{source}}", source_type)
        .replace("{{destination}}", destination_type);

    let source_path = get_import_path(source_dm, project);
    let destination_path = get_import_path(destination_dm, project);

    let source_import_maybe_aliased = if maybe_old_name.is_empty() {
        source_type.to_string()
    } else {
        // funny enough, same in ts and python
        format!("{} as {}", destination_type, source_type)
    };

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
            import_line(
                project.language,
                &source_path,
                &[&source_import_maybe_aliased],
            ),
            import_line(project.language, &destination_path, &[destination_type]),
        )
    };

    function_file_template = function_file_template
        .replace("{{source_import}}", &source_import)
        .replace("{{destination_import}}", &destination_import);

    if let Either::Left(dest_model) = destination_dm {
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

        function_file_template.replace("{{destination_object}}", &destination_object)
    } else {
        let empty = match project.language {
            SupportedLanguages::Typescript => "null",
            SupportedLanguages::Python => "None",
        };
        function_file_template.replace("{{destination_object}}", empty)
    }
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
fn get_import_path(data_model: Either<&DataModel, &str>, project: &Project) -> String {
    match data_model {
        Either::Left(dm) => get_data_model_import_path(dm, project),
        Either::Right(_name) => match project.language {
            SupportedLanguages::Typescript => "datamodels/models".to_string(),
            SupportedLanguages::Python => "app.datamodels.models".to_string(),
        },
    }
}
fn get_data_model_import_path(data_model: &DataModel, project: &Project) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::core::infrastructure::table::Column;
    use crate::framework::languages::SupportedLanguages;
    use crate::project::python_project::PythonProject;
    use crate::project::{LanguageProjectConfig, Project};
    use lazy_static::lazy_static;
    use std::path::PathBuf;

    lazy_static! {
        static ref PROJECT: Project = {
            let manifest_location = env!("CARGO_MANIFEST_DIR");
            Project::new(
                &PathBuf::from(manifest_location).join("tests/test_project"),
                "testing".to_string(),
                SupportedLanguages::Typescript,
            )
        };
    }

    #[test]
    fn test_generate_no_data_model() {
        let result = generate(&PROJECT, Either::Right("Foo"), Either::Right("Bar"));

        println!("{}", result);
        assert_eq!(
            result,
            r#"
// Add your models & start the development server to import these types
import { Foo, Bar } from "datamodels/models";


// The 'run' function transforms Foo data to Bar format.
// For more details on how Moose streaming functions work, see: https://docs.moosejs.com
export default function run(source: Foo): Bar | null {
  return null;
}

"#
        )
    }

    #[test]
    fn test_generate_with_data_model() {
        let result = generate(
            &PROJECT,
            Either::Left(&DataModel {
                columns: to_columns(vec![
                    ("eventId", ColumnType::String, true),
                    ("timestamp", ColumnType::String, false),
                    ("userId", ColumnType::String, false),
                    ("activity", ColumnType::String, false),
                ]),
                name: "UserActivity".to_string(),
                config: Default::default(),
                abs_file_path: PROJECT.data_models_dir().join("models.ts"),
                version: "0.0".to_string(),
            }),
            Either::Left(&DataModel {
                columns: to_columns(vec![
                    ("eventId", ColumnType::String, true),
                    ("timestamp", ColumnType::DateTime, false),
                    ("userId", ColumnType::String, false),
                    ("activity", ColumnType::String, false),
                ]),
                name: "ParsedActivity".to_string(),
                config: Default::default(),
                abs_file_path: PROJECT.data_models_dir().join("models.ts"),
                version: "0.0".to_string(),
            }),
        );

        println!("{}", result);
        assert_eq!(
            result,
            r#"
// Add your models & start the development server to import these types
import { UserActivity, ParsedActivity } from "datamodels/models";


// The 'run' function transforms UserActivity data to ParsedActivity format.
// For more details on how Moose streaming functions work, see: https://docs.moosejs.com
export default function run(source: UserActivity): ParsedActivity | null {
  return {
    eventId: "",
    timestamp: new Date(),
    userId: "",
    activity: "",
  };
}

"#
        )
    }

    #[test]
    fn test_migration() {
        let mut project = PROJECT.clone();
        match project.language_project_config {
            LanguageProjectConfig::Typescript(ref mut t) => {
                t.version = "0.1".to_string();
            }
            LanguageProjectConfig::Python(_) => {}
        }

        let result = generate(
            &project,
            Either::Left(&DataModel {
                columns: to_columns(vec![
                    ("eventId", ColumnType::String, true),
                    ("timestamp", ColumnType::String, false),
                    ("userId", ColumnType::String, false),
                    ("activity", ColumnType::String, false),
                ]),
                name: "UserActivity".to_string(),
                config: Default::default(),
                abs_file_path: project
                    .old_version_location("0.0")
                    .unwrap()
                    .join("models.ts"),
                version: "0.0".to_string(),
            }),
            Either::Left(&DataModel {
                columns: to_columns(vec![
                    ("eventId", ColumnType::String, true),
                    ("timestamp", ColumnType::String, false),
                    ("userId", ColumnType::String, false),
                    ("activity", ColumnType::String, false),
                    ("stuff", ColumnType::String, false),
                ]),
                name: "UserActivity".to_string(),
                config: Default::default(),
                abs_file_path: project.data_models_dir().join("models.ts"),
                version: "0.1".to_string(),
            }),
        );

        println!("{}", result);
        assert_eq!(
            result,
            r#"
// Add your models & start the development server to import these types
import { UserActivity as UserActivityOld } from "versions/0.0/models";
import { UserActivity } from "datamodels/models";

// The 'run' function transforms UserActivityOld data to UserActivity format.
// For more details on how Moose streaming functions work, see: https://docs.moosejs.com
export default function run(source: UserActivityOld): UserActivity | null {
  return { ...source };
}

"#
        )
    }

    #[test]
    fn test_migration_python() {
        let mut project = PROJECT.clone();
        project.language_project_config = LanguageProjectConfig::Python(PythonProject {
            name: "test".to_string(),
            version: "0.1".to_string(),
            dependencies: vec![],
        });
        project.language = SupportedLanguages::Python;

        let result = generate(
            &project,
            Either::Left(&DataModel {
                columns: to_columns(vec![
                    ("eventId", ColumnType::String, true),
                    ("timestamp", ColumnType::String, false),
                    ("userId", ColumnType::String, false),
                    ("activity", ColumnType::String, false),
                ]),
                name: "UserActivity".to_string(),
                config: Default::default(),
                abs_file_path: project
                    .old_version_location("0.0")
                    .unwrap()
                    .join("models.ts"),
                version: "0.0".to_string(),
            }),
            Either::Left(&DataModel {
                columns: to_columns(vec![
                    ("eventId", ColumnType::String, true),
                    ("timestamp", ColumnType::String, false),
                    ("userId", ColumnType::String, false),
                    ("activity", ColumnType::String, false),
                    ("stuff", ColumnType::String, false),
                ]),
                name: "UserActivity".to_string(),
                config: Default::default(),
                abs_file_path: project.data_models_dir().join("models.ts"),
                version: "0.1".to_string(),
            }),
        );

        println!("{}", result);
        assert_eq!(
            result,
            r#"
# Add your models & start the development server to import these types
from v0_0.models import UserActivity as UserActivityOld
from app.datamodels.models import UserActivity
from typing import Callable, Optional
from datetime import datetime

@dataclass
class Flow:
    run: Callable

def flow(activity: UserActivityOld) -> Optional[UserActivity]:
    return UserActivity(
        eventId="",
        timestamp="",
        userId="",
        activity="",
        stuff="",
    )

my_flow = Flow(
    run=flow
)
"#
        )
    }

    fn to_columns(columns: Vec<(&str, ColumnType, bool)>) -> Vec<Column> {
        columns
            .into_iter()
            .map(|(name, data_type, is_key)| Column {
                name: name.to_string(),
                data_type,
                required: true,
                unique: is_key,
                primary_key: is_key,
                default: None,
            })
            .collect()
    }
}
