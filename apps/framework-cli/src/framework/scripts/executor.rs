use std::path::Path;

use anyhow::Result;
use toml;

use super::config::WorkflowConfig;
use crate::framework::{
    languages::SupportedLanguages,
    python::{self, executor::execute_python_workflow},
    typescript::{self, executor::execute_typescript_workflow},
};

use crate::framework::scripts::errors::TemporalExecutionError;

// TODO: Refactor with python executor

#[derive(Debug, thiserror::Error)]
pub enum WorkflowExecutionError {
    #[error("Temporal error: {0}")]
    TemporalError(#[from] TemporalExecutionError),
    #[error("Config error: {0}")]
    ConfigError(String),
}

/// Execute a specific script
pub(crate) async fn execute_workflow(
    language: SupportedLanguages,
    workflow_id: &str,
    execution_path: &Path,
    input: Option<String>,
) -> Result<String, WorkflowExecutionError> {
    let config_path = execution_path.join("config.toml");
    let config_content = std::fs::read_to_string(config_path).map_err(|e| {
        WorkflowExecutionError::ConfigError(format!("Failed to read config.toml: {}", e))
    })?;

    let config: WorkflowConfig = toml::from_str(&config_content).map_err(|e| {
        WorkflowExecutionError::ConfigError(format!("Failed to parse config.toml: {}", e))
    })?;

    match language {
        SupportedLanguages::Python => {
            let run_id =
                execute_python_workflow(workflow_id, execution_path, &config, input).await?;
            Ok(run_id)
        }
        SupportedLanguages::Typescript => {
            let run_id =
                execute_typescript_workflow(workflow_id, execution_path, &config, input).await?;
            Ok(run_id)
        }
    }
}
