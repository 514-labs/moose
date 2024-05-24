use std::path::PathBuf;

use crate::{framework::typescript, infrastructure::stream::redpanda::RedpandaConfig};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FlowError {
    #[error("Failed to load flow files")]
    IoError(#[from] std::io::Error),

    #[error("The flow {file_name} is not supported.")]
    UnsupportedFlowType { file_name: String },

    #[error("Could not fetch the list of topics from Kafka")]
    KafkaError(#[from] rdkafka::error::KafkaError),
}

#[derive(Debug, Clone)]
pub struct Flow {
    pub source_topic: String,
    pub target_topic: String,
    pub executable: PathBuf,
}

pub fn flow_id(source_topic: &str, target_topic: &str) -> String {
    format!("{}_{}", source_topic, target_topic)
}

impl Flow {
    pub fn id(&self) -> String {
        flow_id(&self.source_topic, &self.target_topic)
    }

    pub fn start(&self, redpanda_config: RedpandaConfig) -> Result<Child, FlowError> {
        match &self.executable.extension() {
            Some(ext) if ext.to_str().unwrap() == "ts" => Ok(typescript::flow::run(
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
