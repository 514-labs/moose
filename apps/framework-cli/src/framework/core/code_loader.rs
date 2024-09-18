/// This is a temporary module should be deleted when we switch
/// to core v2
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use async_recursion::async_recursion;
use log::{debug, info};

use crate::{
    framework::data_model::{
        self, is_schema_file,
        model::{DataModel, DataModelSet},
        parser::parse_data_model_file,
        DuplicateModelError,
    },
    infrastructure::olap::{self, clickhouse::model::ClickHouseTable},
    project::{AggregationSet, Project},
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

pub async fn load_framework_objects(project: &Project) -> anyhow::Result<FrameworkObjectVersions> {
    let old_versions = project.old_versions_sorted();
    crawl_schema(project, &old_versions).await
}

#[async_recursion]
pub async fn get_all_framework_objects(
    project: &Project,
    framework_objects: &mut HashMap<String, FrameworkObject>,
    schema_dir: &Path,
    version: &str,
    aggregations: &AggregationSet,
) -> anyhow::Result<()> {
    if schema_dir.is_dir() {
        for entry in std::fs::read_dir(schema_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                debug!("<DCM> Processing directory: {:?}", path);
                get_all_framework_objects(project, framework_objects, &path, version, aggregations)
                    .await?;
            } else if is_schema_file(&path) {
                debug!("<DCM> Processing file: {:?}", path);
                let objects =
                    get_framework_objects_from_schema_file(project, &path, version, aggregations)
                        .await?;
                for fo in objects {
                    DuplicateModelError::try_insert(framework_objects, fo, &path)?;
                }
            }
        }
    }
    Ok(())
}

pub async fn get_framework_objects_from_schema_file(
    project: &Project,
    path: &Path,
    version: &str,
    aggregations: &AggregationSet,
) -> Result<Vec<FrameworkObject>, DataModelError> {
    let framework_objects = parse_data_model_file(path, version, project)?;
    let mut indexed_models = HashMap::new();

    for model in framework_objects.models {
        if aggregations.current_version == version
            && aggregations.names.contains(model.name.clone().trim())
        {
            return Err(DataModelError::Other {
                message: format!(
                    "Model & aggregation {} cannot have the same name",
                    model.name
                ),
            });
        }

        indexed_models.insert(model.name.clone().trim().to_lowercase(), model);
    }

    let data_models_configs = data_model::config::get(
        path,
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
                    println!("Saved config to {:?} - {:?}", data_model.name, data_model.config);
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
    println!(
        "framework_object_mapper datamodel: {:?}, storage: {:?}",
        s.name, s.config.storage.enabled
    );
    let clickhouse_table = if s.config.storage.enabled {
        let table = s.to_table();
        Some(olap::clickhouse::mapper::std_table_to_clickhouse_table(
            &table,
        )?)
    } else {
        None
    };

    let namespace = project.redpanda_config.get_namespace_prefix();
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

async fn crawl_schema(
    project: &Project,
    old_versions: &[String],
) -> anyhow::Result<FrameworkObjectVersions> {
    info!("<DCM> Checking for old version directories...");

    let mut framework_object_versions = FrameworkObjectVersions::new(
        project.cur_version().to_string(),
        project.data_models_dir().clone(),
    );

    let aggregations = AggregationSet {
        current_version: project.cur_version().to_owned(),
        names: project.get_aggregations(),
    };

    for version in old_versions.iter() {
        let path = project.old_version_location(version)?;

        debug!("<DCM> Processing old version directory: {:?}", path);

        let mut framework_objects = HashMap::new();
        get_all_framework_objects(
            project,
            &mut framework_objects,
            &path,
            version,
            &aggregations,
        )
        .await?;

        let schema_version = SchemaVersion {
            base_path: path,
            models: framework_objects,
        };

        framework_object_versions
            .previous_version_models
            .insert(version.clone(), schema_version);
    }

    let schema_dir = project.data_models_dir();

    let aggregations = AggregationSet {
        current_version: project.cur_version().to_owned(),
        names: project.get_aggregations(),
    };

    info!("<DCM> Starting schema directory crawl...");
    let mut framework_objects: HashMap<String, FrameworkObject> = HashMap::new();
    get_all_framework_objects(
        project,
        &mut framework_objects,
        &schema_dir,
        project.cur_version(),
        &aggregations,
    )
    .await?;

    framework_object_versions.current_models = SchemaVersion {
        base_path: schema_dir.clone(),
        models: framework_objects.clone(),
    };

    Ok(framework_object_versions)
}

#[cfg(test)]
mod tests {
    use crate::framework::languages::SupportedLanguages;

    #[tokio::test]
    async fn test_get_all_framework_objects() {
        use super::*;
        let manifest_location = env!("CARGO_MANIFEST_DIR");

        let project = Project::new(
            &PathBuf::from(manifest_location).join("tests/test_project"),
            "testing".to_string(),
            SupportedLanguages::Typescript,
        );

        let mut framework_objects = HashMap::new();
        let aggregations = AggregationSet {
            current_version: "0.0".to_string(),
            names: HashSet::new(),
        };

        let result = get_all_framework_objects(
            &project,
            &mut framework_objects,
            &project
                .data_models_dir()
                .join("separate_dir_to_test_get_all"),
            "0.0",
            &aggregations,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(framework_objects.len(), 2);
    }
}
