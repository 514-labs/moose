use std::collections::HashMap;

use log::info;
use tokio::process::Child;

use crate::{
    framework::{core::infrastructure::function_process::FunctionProcess, python, typescript},
    project::Project,
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
    project: Project,
}

impl FunctionProcessRegistry {
    pub fn new(project: Project) -> Self {
        Self {
            registry: HashMap::new(),
            project,
        }
    }

    pub fn start(
        &mut self,
        function_process: &FunctionProcess,
    ) -> Result<(), FunctionRegistryError> {
        let kafka_config = self.project.redpanda_config.clone();
        let project_path = self.project.project_location.clone();

        let child = if function_process.is_py_function_process() {
            Ok(python::streaming::run(
                &kafka_config,
                &function_process.source_topic,
                &function_process.target_topic,
                &function_process.target_topic_config_json(),
                &function_process.executable,
            )?)
        } else if function_process.is_ts_function_process() {
            Ok(typescript::streaming::run(
                &kafka_config,
                &function_process.source_topic,
                &function_process.target_topic,
                &function_process.target_topic_config_json(),
                &function_process.executable,
                &project_path,
                function_process.parallel_process_count,
                self.project.features.data_model_v2,
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
