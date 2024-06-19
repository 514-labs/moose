use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use walkdir::WalkDir;

use crate::{
    framework::data_model::{
        parser::parse_data_model_file,
        schema::{DataEnum, DataModel},
    },
    project::Project,
};

#[derive(Debug)]
pub struct PrimitiveMap {
    // This probably should not be a top level item and should be nested under the datamodels
    enums: HashSet<DataEnum>,
    datamodels: HashMap<PathBuf, Vec<DataModel>>,
    // TODO add flows
    // TODO add dbblocks
    // TODO add consumption apis
}

#[derive(Debug, thiserror::Error)]
pub enum PrimitiveMapLoadingError {
    #[error("Failure walking the tree")]
    WalkDir(#[from] walkdir::Error),
    #[error("Failed to parse the data model file")]
    DataModelParsing(#[from] crate::framework::data_model::parser::DataModelParsingError),
}

impl PrimitiveMap {
    // Currently limited to the current version - will need to layout previous versions in the future
    pub fn load(project: &Project) -> Result<PrimitiveMap, PrimitiveMapLoadingError> {
        let mut primitive_map = PrimitiveMap {
            enums: HashSet::new(),
            datamodels: HashMap::new(),
        };

        let data_models_dir = project.data_models_dir();
        for res_entry in WalkDir::new(project.app_dir()) {
            let entry = res_entry?;

            if entry.path().starts_with(&data_models_dir) && entry.file_type().is_file() {
                // TODO This doesn't load the configuration - we need to add that
                let file_objects =
                    parse_data_model_file(entry.path(), project.cur_version(), project)?;
                for model in file_objects.models {
                    if let Some(existing_models) = primitive_map.datamodels.get_mut(entry.path()) {
                        existing_models.push(model);
                        continue;
                    } else {
                        primitive_map
                            .datamodels
                            .insert(entry.path().to_path_buf(), vec![model]);
                    }
                }
                for enu in file_objects.enums {
                    primitive_map.enums.insert(enu);
                }
            }
        }

        Ok(primitive_map)
    }

    pub fn data_models_iter(&self) -> impl Iterator<Item = &DataModel> {
        self.datamodels.values().flatten()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{
        framework::{core::primitive_map::PrimitiveMap, languages::SupportedLanguages},
        project::Project,
    };

    #[test]
    #[ignore]
    fn test_load_primitive_map() {
        let project = Project::new(
            Path::new("/Users/nicolas/code/514/test"),
            "test".to_string(),
            SupportedLanguages::Typescript,
        );
        let primitive_map = PrimitiveMap::load(&project);
        println!("{:?}", primitive_map);
        assert!(primitive_map.is_ok());
    }
}
