use walkdir::WalkDir;

use crate::{
    framework::{
        aggregations::model::Aggregation,
        data_model::{
            model::{DataModel, DataModelSet},
            parser::parse_data_model_file,
        },
        flows::{loader::get_all_current_flows, model::Flow},
    },
    project::Project,
};

#[derive(Debug, thiserror::Error)]
pub enum PrimitiveMapLoadingError {
    #[error("Failure walking the tree")]
    WalkDir(#[from] walkdir::Error),
    #[error("Failed to parse the data model file")]
    DataModelParsing(#[from] crate::framework::data_model::parser::DataModelParsingError),

    #[error("Failed to load flows")]
    FlowsLoading(#[from] crate::framework::flows::model::FlowError),
}

#[derive(Debug, Clone, Default)]
pub struct PrimitiveMap {
    pub datamodels: DataModelSet,
    pub functions: Vec<Flow>,

    // We are currently not loading aggregations 1 by 1 in the CLI, we should load them individually to be able
    // to start/stop them individually. Right now we are starting all of them at once through the language specific
    // aggregation runner. We are loading aggregations as 1 unique aggregation as a default.
    pub aggregation: Aggregation, // TODO add consumption apis
}

impl PrimitiveMap {
    // Currently limited to the current version - will need to layout previous versions in the future
    pub async fn load(project: &Project) -> Result<PrimitiveMap, PrimitiveMapLoadingError> {
        let mut primitive_map = PrimitiveMap::default();

        let data_models_dir = project.data_models_dir();

        for res_entry in WalkDir::new(project.app_dir()) {
            let entry = res_entry?;

            if entry.path().starts_with(&data_models_dir) && entry.file_type().is_file() {
                // TODO This doesn't load the configuration - we need to add that
                let file_objects =
                    parse_data_model_file(entry.path(), project.cur_version(), project)?;
                for model in file_objects.models {
                    primitive_map.datamodels.add(model)
                }
            }
        }

        primitive_map.functions = get_all_current_flows(project, &primitive_map.datamodels)
            .await?
            .iter()
            .filter(|flow| flow.executable.extension().unwrap() == "ts")
            .cloned()
            .collect();

        Ok(primitive_map)
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
