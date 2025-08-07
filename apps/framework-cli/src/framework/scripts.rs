use std::path::PathBuf;

use super::languages::SupportedLanguages;

pub mod config;
pub mod executor;
pub mod utils;

use crate::framework::scripts::config::WorkflowConfig;
use crate::infrastructure::orchestration::temporal::TemporalConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};

// A worklfow is a collection of scripts that are executed in order
//
// Workflows are simply a file with a list of scripts within them.
//
// A workflow's name is the name of the folder that contains the scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    name: String,
    path: PathBuf,
    config: WorkflowConfig,
    language: SupportedLanguages,
}

impl Workflow {
    pub fn from_user_code(
        name: String,
        language: SupportedLanguages,
        retries: Option<u32>,
        timeout: Option<String>,
        schedule: Option<String>,
    ) -> Result<Self, anyhow::Error> {
        let config = WorkflowConfig::with_overrides(name.clone(), retries, timeout, schedule);

        Ok(Self {
            name: name.clone(),
            path: PathBuf::from(name.clone()),
            config,
            language,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn config(&self) -> &WorkflowConfig {
        &self.config
    }

    /// Start the workflow execution locally
    pub async fn start(
        &self,
        temporal_config: &TemporalConfig,
        input: Option<String>,
    ) -> Result<String, anyhow::Error> {
        Ok(executor::execute_workflow(
            temporal_config,
            self.language,
            &self.name,
            &self.config,
            &self.path,
            input,
        )
        .await?)
    }
}
