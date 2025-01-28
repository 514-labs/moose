use std::path::Path;

use anyhow::Result;

use crate::framework::{
    languages::SupportedLanguages,
    python::{self, executor::execute_python_workflow},
};

#[derive(Debug, thiserror::Error)]
pub enum WorkflowExecutionError {
    #[error("Temporal error: {0}")]
    TemporalError(#[from] python::executor::WorkflowExecutionError),
}

/// Execute a specific script
pub(crate) async fn execute_workflow(
    language: SupportedLanguages,
    workflow_id: &str,
    execution_path: &Path,
) -> Result<(), WorkflowExecutionError> {
    match language {
        SupportedLanguages::Python => {
            Ok(execute_python_workflow(workflow_id, execution_path).await?)
        }
        _ => todo!("Unsupported language {}", language),
    }
}
