use crate::framework::languages::SupportedLanguages;
use crate::proto::infrastructure_map::OrchestrationWorker as ProtoOrchestrationWorker;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrchestrationWorker {
    // TODO: Add all the workflows and activities here that are available in the worker
    pub supported_language: SupportedLanguages,
}

impl OrchestrationWorker {
    pub fn new(supported_language: SupportedLanguages) -> Self {
        Self { supported_language }
    }

    pub fn id(&self) -> String {
        format!("orchestration_worker_{}", self.supported_language)
    }

    pub fn to_proto(&self) -> ProtoOrchestrationWorker {
        ProtoOrchestrationWorker {
            supported_language: self.supported_language.to_string(),
            special_fields: Default::default(),
        }
    }
}
