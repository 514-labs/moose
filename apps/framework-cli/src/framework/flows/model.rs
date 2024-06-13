use std::{collections::HashMap, path::PathBuf};

use tokio::process::Child;

use crate::{
    framework::{python, typescript},
    infrastructure::stream::redpanda::RedpandaConfig,
    utilities::system::KillProcessError,
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FlowError {
    #[error("Failed to load flow files")]
    IoError(#[from] std::io::Error),

    #[error("The flow {file_name} is not supported.")]
    UnsupportedFlowType { file_name: String },

    #[error("Could not fetch the list of topics from Kafka")]
    KafkaError(#[from] rdkafka::error::KafkaError),

    #[error("Kill process Error")]
    KillProcessError(#[from] KillProcessError),
}

#[derive(Debug, Clone)]
pub struct Flow {
    pub source_topic: String,
    pub target_topic: String,
    pub target_topic_config: HashMap<String, String>,
    pub executable: PathBuf,
}

pub fn flow_id(source_topic: &str, target_topic: &str) -> String {
    format!("{}_{}", source_topic, target_topic)
}

impl Flow {
    pub fn id(&self) -> String {
        flow_id(&self.source_topic, &self.target_topic)
    }

    pub fn target_topic_config_json(&self) -> String {
        serde_json::to_string(&self.target_topic_config).unwrap()
    }

    pub fn start(&self, redpanda_config: RedpandaConfig) -> Result<Child, FlowError> {
        match &self.executable.extension() {
            Some(ext) if ext.to_str().unwrap() == "ts" => Ok(typescript::flow::run(
                redpanda_config,
                &self.source_topic,
                &self.target_topic,
                &self.target_topic_config_json(),
                &self.executable,
            )?),
            Some(ext) if ext.to_str().unwrap() == "py" => Ok(python::flow::run(
                redpanda_config,
                &self.source_topic,
                &self.target_topic,
                &self.executable,
            )?),
            _ => Err(FlowError::UnsupportedFlowType {
                file_name: self
                    .executable
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            }),
        }
    }
}
