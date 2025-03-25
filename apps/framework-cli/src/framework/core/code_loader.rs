/// This is a temporary module should be deleted when we switch
/// to core v2
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use async_recursion::async_recursion;

use crate::{
    framework::data_model::{
        self, is_schema_file,
        model::{DataModel, DataModelSet},
        parser::parse_data_model_file,
        DuplicateModelError,
    },
    infrastructure::olap::{self, clickhouse::model::ClickHouseTable},
    project::Project,
};

use super::{
    infrastructure::table::{ColumnType, DataEnum},
    primitive_map::DataModelError,
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CodeLoadingError {}

#[derive(Debug, Clone)]
pub struct FrameworkObject {
    pub data_model: DataModel,

    // TODO - we should not have infrastructure specific representation here
    // Clickhouse and Kadfka specifics should be abstracted away
    pub table: Option<ClickHouseTable>,
    pub topic: String,
    pub original_file_path: PathBuf,
}

// TODO: save this object somewhere so that we can clean up removed models
// TODO abstract this with an iterator and some helper functions to make the internal state hidden.
// That would enable us to do things like .next() and .peek() on the iterator that are now not possible
#[derive(Debug, Clone)]
pub struct FrameworkObjectVersions {
    pub current_version: String,
    pub current_models: SchemaVersion,
    pub previous_version_models: HashMap<String, SchemaVersion>,
}

impl FrameworkObjectVersions {
    pub fn new(current_version: String, current_schema_directory: PathBuf) -> Self {
        FrameworkObjectVersions {
            current_version,
            current_models: SchemaVersion {
                base_path: current_schema_directory,
                models: HashMap::new(),
            },
            previous_version_models: HashMap::new(),
        }
    }

    pub fn all_versions(&self) -> impl Iterator<Item = &SchemaVersion> {
        std::iter::once(&self.current_models).chain(self.previous_version_models.values())
    }

    // used only when not core_v2
    pub fn to_data_model_set(&self) -> DataModelSet {
        let mut data_model_set = DataModelSet::new();
        for version in self.all_versions() {
            for model in version.models.values() {
                // we're iterating maps, no chance of duplicates
                data_model_set.add(model.data_model.clone()).unwrap();
            }
        }
        data_model_set
    }
}

#[derive(Debug, Clone)]
pub struct SchemaVersion {
    pub base_path: PathBuf,
    pub models: HashMap<String, FrameworkObject>,
}

impl SchemaVersion {
    pub fn get_all_enums(&self) -> Vec<DataEnum> {
        let mut enums = Vec::new();
        for model in self.models.values() {
            for column in &model.data_model.columns {
                if let ColumnType::Enum(data_enum) = &column.data_type {
                    enums.push(data_enum.clone());
                }
            }
        }
        enums
    }

    pub fn get_all_models(&self) -> Vec<DataModel> {
        self.models
            .values()
            .map(|model| model.data_model.clone())
            .collect()
    }
}

// Used by moose-cli import command. This still uses core v1 code
pub async fn load_framework_objects(project: &Project) -> anyhow::Result<FrameworkObjectVersions> {
    let mut framework_object_versions = FrameworkObjectVersions::new(
        project.cur_version().to_string(),
        project.data_models_dir().clone(),
    );

    let schema_dir = project.data_models_dir();

    let mut framework_objects: HashMap<String, FrameworkObject> = HashMap::new();
    get_all_framework_objects(
        project,
        &mut framework_objects,
        &schema_dir,
        project.cur_version().as_str(),
    )
    .await?;

    framework_object_versions.current_models = SchemaVersion {
        base_path: schema_dir.clone(),
        models: framework_objects.clone(),
    };

    Ok(framework_object_versions)
}

// Used by moose-cli import command. This still uses core v1 code
#[async_recursion]
pub async fn get_all_framework_objects(
    project: &Project,
    framework_objects: &mut HashMap<String, FrameworkObject>,
    schema_dir: &Path,
    version: &str,
) -> anyhow::Result<()> {
    if schema_dir.is_dir() {
        for entry in std::fs::read_dir(schema_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                get_all_framework_objects(project, framework_objects, &path, version).await?;
            } else if is_schema_file(&path) {
                let objects =
                    get_framework_objects_from_schema_file(project, &path, version).await?;
                for fo in objects {
                    DuplicateModelError::try_insert(framework_objects, fo, &path)?;
                }
            }
        }
    }
    Ok(())
}

// Used by moose-cli import command. This still uses core v1 code
pub async fn get_framework_objects_from_schema_file(
    project: &Project,
    path: &Path,
    version: &str,
) -> Result<Vec<FrameworkObject>, DataModelError> {
    let framework_objects = parse_data_model_file(path, version, project)?;
    let mut indexed_models = HashMap::new();

    for model in framework_objects.models {
        indexed_models.insert(model.name.clone().trim().to_lowercase(), model);
    }

    let data_models_configs = data_model::config::get(
        path,
        &project.project_location,
        framework_objects
            .enums
            .iter()
            .map(|e| e.name.as_str())
            .collect::<HashSet<&str>>(),
    )
    .await?;

    for (config_variable_name, config) in data_models_configs.iter() {
        let sanitized_config_name = config_variable_name.trim().to_lowercase();
        match sanitized_config_name.strip_suffix("config") {
            Some(config_name_without_suffix) => {
                let data_model_opt = indexed_models.get_mut(config_name_without_suffix);
                if let Some(data_model) = data_model_opt {
                    data_model.config = config.clone();
                } else {
                    return Err(DataModelError::Other {
                        message: format!(
                            "Config with name `{}` does not match any data model. Please make sure that the config variable name matches the pattern: <dataModelName>Config",
                            config_variable_name
                        ),
                    });
                }
            }
            None => {
                return Err(DataModelError::Other { message: format!("Config name exports have to be of the format <dataModelName>Config so that they can be correlated to the proper data model. \n {} is not respecting this pattern", config_variable_name) })
            }
        }
    }

    let mut to_return = Vec::new();
    for model in indexed_models.into_values() {
        to_return.push(framework_object_mapper(project, model, path, version)?);
    }

    Ok(to_return)
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to build internal framework object representation")]
#[non_exhaustive]
pub enum MappingError {
    ClickhouseError(#[from] crate::infrastructure::olap::clickhouse::errors::ClickhouseError),
    TypescriptError(#[from] crate::framework::typescript::generator::TypescriptGeneratorError),
}

pub fn framework_object_mapper(
    project: &Project,
    s: DataModel,
    original_file_path: &Path,
    version: &str,
) -> Result<FrameworkObject, MappingError> {
    let clickhouse_table = if s.config.storage.enabled {
        let table = s.to_table();
        Some(olap::clickhouse::mapper::std_table_to_clickhouse_table(
            &table,
        )?)
    } else {
        None
    };

    let namespace = project.kafka_config.get_namespace_prefix();
    let topic = format!(
        "{}{}_{}",
        namespace,
        s.name.clone(),
        version.replace('.', "_")
    );

    Ok(FrameworkObject {
        data_model: s.clone(),
        table: clickhouse_table,
        topic,
        original_file_path: original_file_path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::languages::SupportedLanguages;
    use crate::project::Project;
    use ctor::ctor;
    use lazy_static::lazy_static;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    #[ctor]
    fn setup() {
        let node_modules_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/test_project")
            .join("node_modules");
        if node_modules_path.exists() {
            fs::remove_dir_all(&node_modules_path).expect("Failed to clean up node_modules");
        }
    }

    fn pnpm_moose_lib(cmd_action: fn(&mut Command) -> &mut Command) {
        let mut cmd = Command::new("pnpm");
        cmd_action(&mut cmd)
            .arg("--filter=moose-lib")
            .current_dir("../../")
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }

    lazy_static! {
        static ref TEST_PROJECT: Project = {
            pnpm_moose_lib(|cmd| cmd.arg("i").arg("--frozen-lockfile"));

            pnpm_moose_lib(|cmd| cmd.arg("run").arg("build"));

            Command::new("npm")
                .arg("i")
                .current_dir("./tests/test_project")
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            Command::new("npm")
                .arg("link")
                .current_dir("../../packages/ts-moose-lib")
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            Command::new("npm")
                .arg("link")
                .arg("@514labs/moose-lib")
                .current_dir("./tests/test_project")
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            Project::new(
                &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/test_project"),
                "testing".to_string(),
                SupportedLanguages::Typescript,
            )
        };
    }

    #[tokio::test]
    #[serial_test::serial(tspc)]
    async fn test_get_all_framework_objects() {
        let mut framework_objects = HashMap::new();

        let result = get_all_framework_objects(
            &TEST_PROJECT,
            &mut framework_objects,
            &TEST_PROJECT
                .data_models_dir()
                .join("separate_dir_to_test_get_all"),
            "0.0",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(framework_objects.len(), 2);
    }
}
