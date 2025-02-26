use anyhow::Result;
use log::info;
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
    let is_local = temporal_url.contains("localhost");
    info!("temporal_url: {} | is_local: {}", temporal_url, is_local);

    let namespace = if is_local {
        DEFAULT_TEMPORTAL_NAMESPACE.to_string()
    } else {
        let domain_name = temporal_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split(':')
            .next()
            .unwrap_or("");

        domain_name
            .strip_suffix(".tmprl.cloud")
            .unwrap_or(domain_name)
            .to_string()
    };

    info!("Using namespace: {}", namespace);

    let channel = if is_local {
        let endpoint = tonic::transport::Channel::from_shared(temporal_url.to_string())
            .map_err(|e| TemporalExecutionError::TimeoutError(e.to_string()))?;

        endpoint.connect().await?
    } else {
        let domain_name = temporal_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split(':')
            .next()
            .unwrap_or("");

        info!("Using domain name for TLS: {}", domain_name);

        let get_env_var = |name: &str| std::env::var(name).unwrap_or_else(|_| "".to_string());
        let ca_cert_path = get_env_var("MOOSE_TEMPORAL_CONFIG__CA_CERT");
        let client_cert_path = get_env_var("MOOSE_TEMPORAL_CONFIG__CLIENT_CERT");
        let client_key_path = get_env_var("MOOSE_TEMPORAL_CONFIG__CLIENT_KEY");

        let client_identity = tonic::transport::Identity::from_pem(
            std::fs::read(client_cert_path)
                .map_err(|e| TemporalExecutionError::TimeoutError(e.to_string()))?,
            std::fs::read(client_key_path)
                .map_err(|e| TemporalExecutionError::TimeoutError(e.to_string()))?,
        );

        let ca_certificate = tonic::transport::Certificate::from_pem(
            std::fs::read(ca_cert_path)
                .map_err(|e| TemporalExecutionError::TimeoutError(e.to_string()))?,
        );

        let tls_config = tonic::transport::ClientTlsConfig::new()
            .identity(client_identity)
            .ca_certificate(ca_certificate)
            .domain_name(domain_name);

        let endpoint = tonic::transport::Channel::from_shared(temporal_url.to_string())
            .map_err(|e| TemporalExecutionError::TimeoutError(e.to_string()))?;

        endpoint.tls_config(tls_config)?.connect().await?
    };

    let mut client = WorkflowServiceClient::new(channel);

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
        namespace: namespace.to_string(),
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
