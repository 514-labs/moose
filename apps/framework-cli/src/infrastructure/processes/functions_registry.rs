use std::{collections::HashMap, path::PathBuf};

use log::info;
use tokio::process::Child;

use crate::{
    framework::{
        core::{
            infrastructure::function_process::FunctionProcess,
            infrastructure_map::InfrastructureMap,
        },
        python, typescript,
    },
    infrastructure::stream::{
        redpanda::models::{RedpandaConfig, RedpandaStreamConfig},
        StreamConfig,
    },
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

    #[error("Topic not found in the infrastructure map")]
    TopicNotFound { topic_id: String },
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
        infra_map: &InfrastructureMap,
        function_process: &FunctionProcess,
    ) -> Result<(), FunctionRegistryError> {
        match (
            infra_map.get_topic_by_id(&function_process.source_topic_id),
            infra_map.get_topic_by_id(&function_process.target_topic_id),
        ) {
            (Some(source_topic), Some(target_topic)) => {
                // TODO This will need to be made generic
                let source_topic = StreamConfig::Redpanda(RedpandaStreamConfig::from_topic(
                    &self.kafka_config,
                    source_topic,
                ));
                let target_topic = StreamConfig::Redpanda(RedpandaStreamConfig::from_topic(
                    &self.kafka_config,
                    target_topic,
                ));

                let child = if function_process.is_py_function_process() {
                    Ok(python::streaming::run(
                        &self.kafka_config,
                        &source_topic,
                        &target_topic,
                        &function_process.executable,
                    )?)
                } else if function_process.is_ts_function_process() {
                    Ok(typescript::streaming::run(
                        &self.kafka_config,
                        &source_topic,
                        &target_topic,
                        &function_process.executable,
                        &self.project_path,
                        function_process.parallel_process_count,
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
            _ => Err(FunctionRegistryError::TopicNotFound {
                topic_id: function_process.source_topic_id.clone(),
            }),
        }
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
