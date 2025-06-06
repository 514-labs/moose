use std::path::PathBuf;

use crate::{
    framework::{data_model::model::DataModel, versions::Version},
    utilities::{
        constants::{PYTHON_FILE_EXTENSION, SQL_FILE_EXTENSION, TYPESCRIPT_FILE_EXTENSION},
        system::KillProcessError,
    },
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FunctionError {
    #[error("Failed to load streaming function files")]
    IoError(#[from] std::io::Error),

    #[error("The streaming function {file_name} is not supported.")]
    UnsupportedFunctionType { file_name: String },

    #[error("Could not fetch the list of topics from Kafka")]
    KafkaError(#[from] rdkafka::error::KafkaError),

    #[error("Kill process Error")]
    KillProcessError(#[from] KillProcessError),
}

#[derive(Debug, Clone)]
pub struct StreamingFunction {
    // The name used here is the name of the file that contains the function
    // since we have the current assumption that there is 1 function per file
    // we can use the file name as the name of the function.
    // We don't currently do checks across streaming functions for unicities but we should
    pub name: String,

    pub source_data_model: DataModel,
    pub target_data_model: Option<DataModel>,

    pub executable: PathBuf,

    pub version: Version,
}

impl StreamingFunction {
    // Should the version of the data models be included in the id?
    pub fn id(&self) -> String {
        let base_id = format!("{}_{}", self.name, self.source_data_model.name,);

        if let Some(target) = &self.target_data_model {
            format!("{}_{}_{}", base_id, target.name, self.version)
        } else {
            format!("{}_{}", base_id, self.version)
        }
    }

    pub fn is_ts(&self) -> bool {
        self.executable.extension().unwrap().to_str().unwrap() == TYPESCRIPT_FILE_EXTENSION
    }

    pub fn is_py(&self) -> bool {
        self.executable.extension().unwrap().to_str().unwrap() == PYTHON_FILE_EXTENSION
    }

    pub fn is_migration(&self) -> bool {
        match &self.target_data_model {
            Some(target) => {
                self.source_data_model.version != target.version
                    && self.executable.extension().unwrap().to_str().unwrap() != SQL_FILE_EXTENSION
            }
            None => false,
        }
    }
}
