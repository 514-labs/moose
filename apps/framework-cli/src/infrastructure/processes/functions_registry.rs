use std::collections::HashMap;

use log::info;
use tokio::process::Child;

use crate::{
    framework::{
        core::infrastructure::{function_process::FunctionProcess, topic::Topic},
        flows::model::Flow,
        python, typescript,
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
}

impl FunctionProcessRegistry {
    pub fn new(kafka_config: RedpandaConfig) -> Self {
        Self {
            registry: HashMap::new(),
            kafka_config,
        }
    }

    pub fn start(
        &mut self,
        function_process: &FunctionProcess,
    ) -> Result<(), FunctionRegistryError> {
        let child = if function_process.is_py_function_process() {
            Ok(python::flow::run(
                &self.kafka_config,
                &function_process.source_topic,
                &function_process.target_topic,
                &function_process.target_topic_config_json(),
                &function_process.executable,
            )?)
        } else if function_process.is_ts_function_process() {
            Ok(typescript::flow::run(
                &self.kafka_config,
                &function_process.source_topic,
                &function_process.target_topic,
                &function_process.target_topic_config_json(),
                &function_process.executable,
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

    // This is a legacy method that takes in directly a flow to accomodate the current way
    // of spinning up flows
    pub fn start_all(
        &mut self,
        flows: &[Flow],
        topics: &[String],
    ) -> Result<(), FunctionRegistryError> {
        for flow in flows {
            if flow.is_flow_migration() {
                let (source_topic, target_topic) = Topic::from_migration_function(flow);
                let function_process =
                    FunctionProcess::from_migration_functon(flow, &source_topic, &target_topic);
                self.start(&function_process)?;
            } else {
                self.start(&FunctionProcess::from_functon(flow, topics))?;
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
