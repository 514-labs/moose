use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use toml;

use super::config::WorkflowConfig;
use crate::framework::{
    languages::SupportedLanguages,
    scripts::utils::{parse_schedule, parse_timeout_to_seconds, TemporalExecutionError},
};

use temporal_sdk_core::protos::temporal::api::common::v1::{
    Payload, Payloads, RetryPolicy, WorkflowType,
};
use temporal_sdk_core::protos::temporal::api::enums::v1::{
    TaskQueueKind, WorkflowIdConflictPolicy, WorkflowIdReusePolicy,
};

use temporal_sdk_core::protos::temporal::api::taskqueue::v1::TaskQueue;
use temporal_sdk_core::protos::temporal::api::workflowservice::v1::workflow_service_client::WorkflowServiceClient;
use temporal_sdk_core::protos::temporal::api::workflowservice::v1::StartWorkflowExecutionRequest;

const WORKFLOW_TYPE: &str = "ScriptWorkflow";
const DEFAULT_TEMPORTAL_NAMESPACE: &str = "default";
const PYTHON_TASK_QUEUE: &str = "python-script-queue";
const TYPESCRIPT_TASK_QUEUE: &str = "typescript-script-queue";
const MOOSE_CLI_IDENTITY: &str = "moose-cli";

#[derive(Debug, thiserror::Error)]
pub enum WorkflowExecutionError {
    #[error("Temporal error: {0}")]
    TemporalError(#[from] TemporalExecutionError),
    #[error("Config error: {0}")]
    ConfigError(String),
}

/// Execute a specific script
pub(crate) async fn execute_workflow(
    temporal_url: &str,
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
            let run_id = execute_workflow_for_language(
                temporal_url,
                workflow_id,
                execution_path,
                &config,
                input,
                PYTHON_TASK_QUEUE,
            )
            .await?;
            Ok(run_id)
        }
        SupportedLanguages::Typescript => {
            let run_id = execute_workflow_for_language(
                temporal_url,
                workflow_id,
                execution_path,
                &config,
                input,
                TYPESCRIPT_TASK_QUEUE,
            )
            .await?;
            Ok(run_id)
        }
    }
}

async fn execute_workflow_for_language(
    temporal_url: &str,
    workflow_id: &str,
    execution_path: &Path,
    config: &WorkflowConfig,
    input: Option<String>,
    task_queue_name: &str,
) -> Result<String, TemporalExecutionError> {
    let endpoint = tonic::transport::Endpoint::from_shared(temporal_url.to_string())?;
    let mut client = WorkflowServiceClient::connect(endpoint).await?;

    let mut payloads = vec![Payload {
        metadata: HashMap::from([(
            String::from("encoding"),
            String::from("json/plain").into_bytes(),
        )]),
        data: serde_json::to_string(execution_path)
            .unwrap()
            .as_bytes()
            .to_vec(),
    }];

    if let Some(data) = input {
        let input_data_json_value: serde_json::Value = serde_json::from_str(&data).unwrap();
        let serialized_data = serde_json::to_string(&input_data_json_value).unwrap();
        payloads.push(Payload {
            metadata: HashMap::from([(
                String::from("encoding"),
                String::from("json/plain").into_bytes(),
            )]),
            data: serialized_data.as_bytes().to_vec(),
        });
    }

    // Test workflow executor from consumption api if this changes significantly
    let request = tonic::Request::new(StartWorkflowExecutionRequest {
        namespace: DEFAULT_TEMPORTAL_NAMESPACE.to_string(),
        workflow_id: workflow_id.to_string(),
        workflow_type: Some(WorkflowType {
            name: WORKFLOW_TYPE.to_string(),
        }),
        task_queue: Some(TaskQueue {
            name: task_queue_name.to_string(),
            kind: TaskQueueKind::Normal as i32,
            normal_name: task_queue_name.to_string(),
        }),
        input: Some(Payloads { payloads }),
        workflow_run_timeout: Some(prost_wkt_types::Duration {
            seconds: parse_timeout_to_seconds(&config.timeout)?,
            nanos: 0,
        }),
        identity: MOOSE_CLI_IDENTITY.to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        workflow_id_reuse_policy: WorkflowIdReusePolicy::AllowDuplicate as i32,
        retry_policy: Some(RetryPolicy {
            maximum_attempts: config.retries as i32,
            ..Default::default()
        }),
        cron_schedule: parse_schedule(&config.schedule),
        workflow_id_conflict_policy: WorkflowIdConflictPolicy::Unspecified as i32,
        request_eager_execution: false,
        ..Default::default()
    });

    let response = client.start_workflow_execution(request).await?;
    let run_id = response.into_inner().run_id;
    Ok(run_id)
}
