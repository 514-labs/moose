use std::{collections::HashMap, path::PathBuf};

use log::info;
use tokio::process::Child;

use crate::{
    framework::{
        core::infrastructure::{function_process::FunctionProcess, topic::Topic},
        python,
        streaming::model::StreamingFunction,
        typescript,
    },
    infrastructure::stream::redpanda::RedpandaConfig,
    utilities::system::{kill_child, KillProcessError},
};

#[derive(Debug, thiserror::Error)]
pub enum FunctionRegistryError {
    #[error("Failed to load function_process files")]
    IO(#[from] std::io::Error),

    #[error("Could not kill the process.")]
    KillProcess(#[from] KillProcessError),

    #[error("Cannot run function_process {file_name}. Unsupported function_process type")]
    UnsupportedFunctionLanguage { file_name: String },
}

pub struct FunctionProcessRegistry {
    registry: HashMap<String, Child>,
    kafka_config: RedpandaConfig,
    project_path: PathBuf,
}

impl FunctionProcessRegistry {
    pub fn new(kafka_config: RedpandaConfig, project_path: PathBuf) -> Self {
        Self {
            registry: HashMap::new(),
            kafka_config,
            project_path,
        }
    }

    pub fn start(
        &mut self,
        function_process: &FunctionProcess,
    ) -> Result<(), FunctionRegistryError> {
        let child = if function_process.is_py_function_process() {
            Ok(python::streaming::run(
                &self.kafka_config,
                &function_process.source_topic,
                &function_process.target_topic,
                &function_process.target_topic_config_json(),
                &function_process.executable,
            )?)
        } else if function_process.is_ts_function_process() {
            Ok(typescript::streaming::run(
                &self.kafka_config,
                &function_process.source_topic,
                &function_process.target_topic,
                &function_process.target_topic_config_json(),
                &function_process.executable,
                &self.project_path,
            )?)
        } else {
            Err(FunctionRegistryError::UnsupportedFunctionLanguage {
                file_name: function_process
                    .executable
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            })
        }?;

        self.registry.insert(function_process.id(), child);

        Ok(())
    }

    // This is a legacy method that takes in directly a streaming function
    // to accommodate the current way of spinning up streaming functions
    pub fn start_all(
        &mut self,
        functions: &[StreamingFunction],
        topics: &[String],
    ) -> Result<(), FunctionRegistryError> {
        for streaming_function in functions {
            if streaming_function.is_migration() {
                let (source_topic, target_topic) =
                    Topic::from_migration_function(streaming_function);
                let function_process = FunctionProcess::from_migration_function(
                    streaming_function,
                    &source_topic,
                    &target_topic.unwrap(),
                );
                self.start(&function_process)?;
            } else {
                self.start(&FunctionProcess::from_function(streaming_function, topics))?;
            }
        }

        Ok(())
    }

    pub async fn stop(
        &mut self,
        function_process: &FunctionProcess,
    ) -> Result<(), FunctionRegistryError> {
        info!("Stopping function process {:?}...", function_process.id());

        let id = &function_process.id();
        if let Some(running_function_process) = self.registry.get_mut(id) {
            kill_child(running_function_process).await?;
            self.registry.remove(id);
        }

        Ok(())
    }

    pub async fn stop_all(&mut self) -> Result<(), FunctionRegistryError> {
        for (id, running_function_process) in self.registry.iter_mut() {
            info!("Stopping function_process {:?}...", id);
            kill_child(running_function_process).await?;
        }

        self.registry.clear();

        Ok(())
    }
}
