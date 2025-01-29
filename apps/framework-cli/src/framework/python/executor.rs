//! # Executes Python code in a subprocess.
//! This module provides a Python executor that can run Python code in a subprocess

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;

use crate::utilities::constants::{CLI_INTERNAL_VERSIONS_DIR, CLI_PROJECT_INTERNAL_DIR};

use temporal_sdk_core::protos::temporal::api::common::v1::{Payload, Payloads, WorkflowType};
use temporal_sdk_core::protos::temporal::api::enums::v1::{
    TaskQueueKind, WorkflowIdConflictPolicy, WorkflowIdReusePolicy,
};

use temporal_sdk_core::protos::temporal::api::taskqueue::v1::TaskQueue;
use temporal_sdk_core::protos::temporal::api::workflowservice::v1::workflow_service_client::WorkflowServiceClient;
use temporal_sdk_core::protos::temporal::api::workflowservice::v1::StartWorkflowExecutionRequest;
use tokio::process::{Child, Command};

pub enum PythonSerializers {
    FrameworkObjectSerializer,
    ProjectObjectSerializer,
}

impl PythonSerializers {
    pub fn get_path(&self) -> &str {
        match self {
            PythonSerializers::FrameworkObjectSerializer => {
                "src/framework/python/scripts/framework_object_serializer.py"
            }
            PythonSerializers::ProjectObjectSerializer => {
                "src/framework/python/scripts/project_object_serializer.py"
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PythonProgram {
    StreamingFunctionRunner { args: Vec<String> },
    BlocksRunner { args: Vec<String> },
    ConsumptionRunner { args: Vec<String> },
    LoadApiParam { args: Vec<String> },
    OrchestrationWorker { args: Vec<String> },
}

pub static STREAMING_FUNCTION_RUNNER: &str = include_str!("wrappers/streaming_function_runner.py");
pub static BLOCKS_RUNNER: &str = include_str!("wrappers/blocks_runner.py");
pub static CONSUMPTION_RUNNER: &str = include_str!("wrappers/consumption_runner.py");
pub static LOAD_API_PARAMS: &str = include_str!("wrappers/load_api_params.py");
pub static ORCHESTRATION_WORKER: &str = include_str!("wrappers/scripts/worker-main.py");

const PYTHON_PATH: &str = "PYTHONPATH";
fn python_path_with_version() -> String {
    let mut paths = std::env::var(PYTHON_PATH).unwrap_or_else(|_| String::from(""));
    if !paths.is_empty() {
        paths.push(':');
    }
    paths.push_str(CLI_PROJECT_INTERNAL_DIR);
    paths.push('/');
    paths.push_str(CLI_INTERNAL_VERSIONS_DIR);

    paths.push(':');
    paths.push_str(CLI_PROJECT_INTERNAL_DIR);

    paths
}

/// Executes a Python program in a subprocess
pub fn run_python_program(program: PythonProgram) -> Result<Child, std::io::Error> {
    let (get_args, program_string) = match program.clone() {
        PythonProgram::StreamingFunctionRunner { args } => (args, STREAMING_FUNCTION_RUNNER),
        PythonProgram::BlocksRunner { args } => (args, BLOCKS_RUNNER),
        PythonProgram::ConsumptionRunner { args } => (args, CONSUMPTION_RUNNER),
        PythonProgram::LoadApiParam { args } => (args, LOAD_API_PARAMS),
        PythonProgram::OrchestrationWorker { args } => (args, ORCHESTRATION_WORKER),
    };

    Command::new("python3")
        .env(PYTHON_PATH, python_path_with_version())
        .arg("-u")
        .arg("-c")
        .arg(program_string)
        .args(get_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub async fn run_python_file(path: &Path, env: &[(&str, &str)]) -> Result<Child, std::io::Error> {
    let mut command = Command::new("python3");

    command.env(PYTHON_PATH, python_path_with_version());
    for (key, val) in env {
        command.env(key, val);
    }

    command
        .arg("-u")
        .arg(path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
}

pub fn add_optional_arg(args: &mut Vec<String>, flag: &str, value: &Option<String>) {
    if let Some(val) = value {
        args.push(flag.to_string());
        args.push(val.to_string());
    }
}

const WORKFLOW_TYPE: &str = "ScriptWorkflow";
const DEFAULT_TEMPORTAL_NAMESPACE: &str = "default";
const PYTHON_TASK_QUEUE: &str = "python-script-queue";
const MOOSE_CLI_IDENTITY: &str = "moose-cli";

#[derive(Debug, thiserror::Error)]
pub enum WorkflowExecutionError {
    #[error("Temportal connection error: {0}")]
    TemporalConnectionError(#[from] tonic::transport::Error),

    #[error("Temportal client error: {0}")]
    TemporalClientError(#[from] tonic::Status),
}

/// Parses various schedule formats into a valid Temporal cron expression
///
/// # Arguments
/// * `schedule` - Optional string containing the schedule format
///
/// # Returns
/// A String containing the parsed cron expression or empty string if invalid
///
/// # Formats Supported
/// * Standard cron expressions (e.g., "* * * * *")
/// * Interval notation (e.g., "*/5 * * * *")
/// * Simple duration formats:
///   - "5m" → "*/5 * * * *" (every 5 minutes)
///   - "2h" → "0 */2 * * *" (every 2 hours)
///
/// Falls back to empty string (no schedule) if format is invalid
fn parse_schedule(schedule: Option<String>) -> String {
    schedule
        .filter(|s| !s.is_empty())
        .map(|s| match s.as_str() {
            // Handle interval-based formats
            s if s.contains('/') => s.to_string(),
            // Handle standard cron expressions
            s if s.contains('*') || s.contains(' ') => s.to_string(),
            // Convert simple duration to cron (e.g., "5m" -> "*/5 * * * *")
            s if s.ends_with('m') => {
                let mins = s.trim_end_matches('m');
                format!("*/{} * * * *", mins)
            }
            s if s.ends_with('h') => {
                let hours = s.trim_end_matches('h');
                format!("0 */{} * * *", hours)
            }
            // Default to original string if format is unrecognized
            s => s.to_string(),
        })
        .unwrap_or_default()
}

pub async fn execute_python_workflow(
    workflow_id: &str,
    execution_path: &Path,
    schedule: Option<String>,
) -> Result<(), WorkflowExecutionError> {
    let endpoint = tonic::transport::Endpoint::from_static("http://localhost:7233");
    let mut client = WorkflowServiceClient::connect(endpoint).await?;

    let request = tonic::Request::new(StartWorkflowExecutionRequest {
        namespace: DEFAULT_TEMPORTAL_NAMESPACE.to_string(),
        workflow_id: workflow_id.to_string(),
        workflow_type: Some(WorkflowType {
            name: WORKFLOW_TYPE.to_string(),
        }),
        task_queue: Some(TaskQueue {
            name: PYTHON_TASK_QUEUE.to_string(),
            kind: TaskQueueKind::Normal as i32,
            normal_name: PYTHON_TASK_QUEUE.to_string(),
        }),
        input: Some(Payloads {
            payloads: vec![Payload {
                metadata: HashMap::from([(
                    String::from("encoding"),
                    String::from("json/plain").into_bytes(),
                )]),
                data: serde_json::to_string(execution_path)
                    .unwrap()
                    .as_bytes()
                    .to_vec(),
            }],
        }),
        workflow_execution_timeout: None,
        workflow_run_timeout: None,
        workflow_task_timeout: None,
        identity: MOOSE_CLI_IDENTITY.to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        search_attributes: None,
        header: None,
        workflow_id_reuse_policy: WorkflowIdReusePolicy::AllowDuplicate as i32,
        retry_policy: None,
        cron_schedule: parse_schedule(schedule),
        memo: None,
        workflow_id_conflict_policy: WorkflowIdConflictPolicy::Unspecified as i32,
        request_eager_execution: false,
        continued_failure: None,
        last_completion_result: None,
        workflow_start_delay: None,
        completion_callbacks: vec![],
        user_metadata: None,
        links: vec![],
    });

    client.start_workflow_execution(request).await?;
    Ok(())
}

// TESTs
#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::infrastructure::stream::redpanda::RedpandaConfig;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_run_python_flow_runner_program() {
        let redpanda_config = RedpandaConfig::default();
        let source_topic = "UserActivity_0_0";
        let target_topic = "ParsedActivity_0_0";
        let flow_path = Path::new(
            "/Users/timdelisle/Dev/igloo-stack/apps/framework-cli/tests/python/flows/valid",
        );

        let program = PythonProgram::StreamingFunctionRunner {
            args: vec![
                source_topic.to_string(),
                target_topic.to_string(),
                flow_path.to_str().unwrap().to_string(),
                redpanda_config.broker,
            ],
        };

        let child = run_python_program(program).unwrap();
        let output = child.wait_with_output().await.unwrap();

        //print output stdout and stderr
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}
