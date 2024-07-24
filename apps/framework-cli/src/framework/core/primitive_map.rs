use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use walkdir::WalkDir;

use crate::{
    framework::{
        aggregations::model::Aggregation,
        consumption::model::Consumption,
        data_model::{
            self,
            config::ModelConfigurationError,
            model::{DataModel, DataModelSet},
            parser::DataModelParsingError,
        },
        streaming::{loader::get_all_current_streaming_functions, model::StreamingFunction},
    },
    project::Project,
};

use super::code_loader::MappingError;

#[derive(Debug, thiserror::Error)]
pub enum PrimitiveMapLoadingError {
    #[error("Failure walking the tree")]
    WalkDir(#[from] walkdir::Error),
    #[error("Failed to parse the data model file")]
    DataModel(#[from] DataModelError),

    #[error("Failed to load functions")]
    FunctionsLoading(#[from] crate::framework::streaming::model::FunctionError),
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to get the Data Model configuration")]
#[non_exhaustive]
pub enum DataModelError {
    Configuration(#[from] ModelConfigurationError),
    Parsing(#[from] DataModelParsingError),
    Mapping(#[from] MappingError),

    #[error("{message}")]
    Other {
        message: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct PrimitiveMap {
    pub datamodels: DataModelSet,
    pub functions: Vec<StreamingFunction>,

    // We are currently not loading aggregations 1 by 1 in the CLI, we should load them individually to be able
    // to start/stop them individually. Right now we are starting all of them at once through the language specific
    // aggregation runner. We are loading aggregations as 1 unique aggregation as a default.
    pub aggregation: Aggregation,

    // We are currrently not loading indiviual consumption endpoints in the CLI and we probably will not need to
    // Since this is a local webserver without side effects, keepting track of what is up and running is not necessary
    // it just needs to be restarted when something in its dependency tree changes.
    // We might want to try and load the full map of consumption endpoints in the future to be able to display thgat
    // to the user.
    pub consumption: Consumption,
}

impl PrimitiveMap {
    // Currently limited to the current version - will need to layout previous versions in the future
    pub async fn load(project: &Project) -> Result<PrimitiveMap, PrimitiveMapLoadingError> {
        let mut primitive_map = PrimitiveMap::default();

        let data_models_dir = project.data_models_dir();

        for res_entry in WalkDir::new(project.app_dir()) {
            let entry = res_entry?;

            if entry.path().starts_with(&data_models_dir) && entry.file_type().is_file() {
                for model in PrimitiveMap::load_data_model(project, entry.path()).await? {
                    primitive_map.datamodels.add(model)
                }
            }
        }

        // TODO Add validation that aggregations and data model names do not overlap

        primitive_map.functions =
            get_all_current_streaming_functions(project, &primitive_map.datamodels)
                .await?
                .iter()
                .filter(|func| func.executable.extension().unwrap() == "ts")
                .cloned()
                .collect();

        Ok(primitive_map)
    }

    /**
     * Loads the data models from the given path and their configurations.
     */
    async fn load_data_model(
        project: &Project,
        path: &Path,
    ) -> Result<Vec<DataModel>, DataModelError> {
        let version = project.cur_version();
        let file_objects = data_model::parser::parse_data_model_file(path, version, project)?;
        let mut indexed_models: HashMap<String, DataModel> = HashMap::new();

        for model in file_objects.models {
            indexed_models.insert(model.name.clone().trim().to_lowercase(), model);
        }

        let data_models_configs = data_model::config::get(
            path,
            file_objects
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

        Ok(indexed_models.values().cloned().collect())
    }

    pub fn data_models_iter(&self) -> impl Iterator<Item = &DataModel> {
        self.datamodels.iter()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        framework::{core::primitive_map::PrimitiveMap, languages::SupportedLanguages},
        project::Project,
    };

    #[tokio::test]
    #[ignore]
    async fn test_load_primitive_map() {
        let project = Project::new(
            Path::new("/Users/nicolas/code/514/test"),
            "test".to_string(),
            SupportedLanguages::Typescript,
        );
        let primitive_map = PrimitiveMap::load(&project).await;
        println!("{:?}", primitive_map);
        assert!(primitive_map.is_ok());
    }
}
