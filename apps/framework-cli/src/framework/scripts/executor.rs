use anyhow::Result;
use log::info;
use std::collections::HashMap;
use std::path::Path;
use toml;

use super::config::WorkflowConfig;
use crate::framework::{
    languages::SupportedLanguages,
    scripts::utils::{
        get_temporal_domain_name, get_temporal_namespace, parse_schedule, parse_timeout_to_seconds,
        TemporalExecutionError,
    },
};
use crate::infrastructure::orchestration::temporal::TemporalConfig;
use crate::infrastructure::orchestration::temporal_client::TemporalClientManager;
use temporal_sdk_core::protos::temporal::api::common::v1::{
    Payload, Payloads, RetryPolicy, WorkflowType,
};
use temporal_sdk_core::protos::temporal::api::enums::v1::{
    TaskQueueKind, WorkflowIdConflictPolicy, WorkflowIdReusePolicy,
};

use temporal_sdk_core::protos::temporal::api::taskqueue::v1::TaskQueue;
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

struct WorkflowExecutionParams<'a> {
    temporal_config: &'a TemporalConfig,
    workflow_id: &'a str,
    execution_path: &'a Path,
    config: &'a WorkflowConfig,
    input: Option<String>,
    task_queue_name: &'a str,
}

/// Execute a specific script
pub(crate) async fn execute_workflow(
    temporal_config: &TemporalConfig,
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
            let params = WorkflowExecutionParams {
                temporal_config,
                workflow_id,
                execution_path,
                config: &config,
                input,
                task_queue_name: PYTHON_TASK_QUEUE,
            };
            let run_id = execute_workflow_for_language(params).await?;
            Ok(run_id)
        }
        SupportedLanguages::Typescript => {
            let params = WorkflowExecutionParams {
                temporal_config,
                workflow_id,
                execution_path,
                config: &config,
                input,
                task_queue_name: TYPESCRIPT_TASK_QUEUE,
            };
            let run_id = execute_workflow_for_language(params).await?;
            Ok(run_id)
        }
    }
}

async fn execute_workflow_for_language(
    params: WorkflowExecutionParams<'_>,
) -> Result<String, TemporalExecutionError> {
    let client_manager = TemporalClientManager::new(params.temporal_config);

    let temporal_url = params.temporal_config.temporal_url_with_scheme();
    let domain_name = get_temporal_domain_name(&temporal_url);
    let namespace = if temporal_url.contains("localhost") {
        DEFAULT_TEMPORTAL_NAMESPACE.to_string()
    } else {
        get_temporal_namespace(domain_name)
    };

    info!("Using namespace: {}", namespace);

    client_manager
        .execute(|mut client| async move {
            let request = create_workflow_execution_request(namespace, &params)?;
            client
                .start_workflow_execution(request)
                .await
                .map_err(|e| anyhow::Error::msg(e.to_string()))
        })
        .await
        .map_err(|e| TemporalExecutionError::TemporalClientError(e.to_string()))
}

fn create_workflow_execution_request(
    namespace: String,
    params: &WorkflowExecutionParams<'_>,
) -> Result<StartWorkflowExecutionRequest, TemporalExecutionError> {
    let mut payloads = vec![Payload {
        metadata: HashMap::from([(
            String::from("encoding"),
            String::from("json/plain").into_bytes(),
        )]),
        data: serde_json::to_string(params.execution_path)
            .unwrap()
            .as_bytes()
            .to_vec(),
    }];

    if let Some(data) = &params.input {
        let input_data_json_value: serde_json::Value = serde_json::from_str(data).unwrap();
        let serialized_data = serde_json::to_string(&input_data_json_value).unwrap();
        payloads.push(Payload {
            metadata: HashMap::from([(
                String::from("encoding"),
                String::from("json/plain").into_bytes(),
            )]),
            data: serialized_data.as_bytes().to_vec(),
        });
    }

    Ok(StartWorkflowExecutionRequest {
        namespace,
        workflow_id: params.workflow_id.to_string(),
        workflow_type: Some(WorkflowType {
            name: WORKFLOW_TYPE.to_string(),
        }),
        task_queue: Some(TaskQueue {
            name: params.task_queue_name.to_string(),
            kind: TaskQueueKind::Normal as i32,
            normal_name: params.task_queue_name.to_string(),
        }),
        input: Some(Payloads { payloads }),
        workflow_run_timeout: Some(prost_wkt_types::Duration {
            seconds: parse_timeout_to_seconds(&params.config.timeout)?,
            nanos: 0,
        }),
        identity: MOOSE_CLI_IDENTITY.to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        workflow_id_reuse_policy: WorkflowIdReusePolicy::AllowDuplicate as i32,
        retry_policy: Some(RetryPolicy {
            maximum_attempts: params.config.retries as i32,
            ..Default::default()
        }),
        cron_schedule: parse_schedule(&params.config.schedule),
        workflow_id_conflict_policy: WorkflowIdConflictPolicy::Unspecified as i32,
        request_eager_execution: false,
        ..Default::default()
    })
}
