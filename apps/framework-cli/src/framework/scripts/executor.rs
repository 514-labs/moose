use anyhow::Result;
use log::info;
use std::collections::HashMap;
use std::path::Path;
use toml;

use super::config::WorkflowConfig;
use super::utils::{get_temporal_domain_name, get_temporal_namespace};
use crate::framework::{
    languages::SupportedLanguages,
    scripts::utils::{parse_schedule, parse_timeout_to_seconds, TemporalExecutionError},
};
use crate::infrastructure::orchestration::temporal::{
    get_temporal_client, get_temporal_client_api_key, get_temporal_client_mtls,
    MOOSE_TEMPORAL_CONFIG__API_KEY, MOOSE_TEMPORAL_CONFIG__CA_CERT,
    MOOSE_TEMPORAL_CONFIG__CLIENT_CERT, MOOSE_TEMPORAL_CONFIG__CLIENT_KEY,
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

struct WorkflowExecutionParams<'a> {
    temporal_url: &'a str,
    workflow_id: &'a str,
    execution_path: &'a Path,
    config: &'a WorkflowConfig,
    input: Option<String>,
    task_queue_name: &'a str,
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
            let params = WorkflowExecutionParams {
                temporal_url,
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
                temporal_url,
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
    let is_local = params.temporal_url.contains("localhost");
    info!(
        "temporal_url: {} | is_local: {}",
        params.temporal_url, is_local
    );

    if is_local {
        let namespace = DEFAULT_TEMPORTAL_NAMESPACE.to_string();
        info!("Using namespace: {}", namespace);

        let mut client = get_temporal_client(params.temporal_url)
            .await
            .map_err(|e| TemporalExecutionError::AuthenticationError(e.to_string()))?;

        start_workflow_execution_with_channel(&mut client, namespace, params).await
    } else if !MOOSE_TEMPORAL_CONFIG__CA_CERT.is_empty()
        && !MOOSE_TEMPORAL_CONFIG__CLIENT_CERT.is_empty()
        && !MOOSE_TEMPORAL_CONFIG__CLIENT_KEY.is_empty()
    {
        let domain_name = get_temporal_domain_name(params.temporal_url);
        let namespace = get_temporal_namespace(domain_name);
        info!(
            "mTLS using domain name: {} | namespace: {}",
            domain_name, namespace
        );

        let mut client = get_temporal_client_mtls(params.temporal_url)
            .await
            .map_err(|e| TemporalExecutionError::AuthenticationError(e.to_string()))?;

        start_workflow_execution_with_channel(&mut client, namespace, params).await
    } else if !MOOSE_TEMPORAL_CONFIG__CA_CERT.is_empty()
        && !MOOSE_TEMPORAL_CONFIG__API_KEY.is_empty()
    {
        let domain_name = get_temporal_domain_name(params.temporal_url);
        let namespace = get_temporal_namespace(domain_name);
        info!(
            "api-key using domain name: {} | namespace: {}",
            domain_name, namespace
        );

        let mut client = get_temporal_client_api_key(params.temporal_url)
            .await
            .map_err(|e| TemporalExecutionError::AuthenticationError(e.to_string()))?;

        start_workflow_execution_with_interceptor(&mut client, namespace, params).await
    } else {
        Err(TemporalExecutionError::AuthenticationError(
            "No authentication credentials provided for Temporal.".to_string(),
        ))
    }
}

async fn start_workflow_execution_with_channel(
    client: &mut WorkflowServiceClient<tonic::transport::Channel>,
    namespace: String,
    params: WorkflowExecutionParams<'_>,
) -> Result<String, TemporalExecutionError> {
    let request = tonic::Request::new(create_workflow_execution_request(namespace, &params)?);
    let response = client.start_workflow_execution(request).await?;
    let run_id = response.into_inner().run_id;
    Ok(run_id)
}

async fn start_workflow_execution_with_interceptor<I>(
    client: &mut WorkflowServiceClient<
        tonic::service::interceptor::InterceptedService<tonic::transport::Channel, I>,
    >,
    namespace: String,
    params: WorkflowExecutionParams<'_>,
) -> Result<String, TemporalExecutionError>
where
    I: tonic::service::Interceptor,
{
    let request = tonic::Request::new(create_workflow_execution_request(namespace, &params)?);
    let response = client.start_workflow_execution(request).await?;
    let run_id = response.into_inner().run_id;
    Ok(run_id)
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
