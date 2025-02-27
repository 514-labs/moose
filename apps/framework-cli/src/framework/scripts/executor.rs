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

struct WorkflowExecutionParams<'a> {
    temporal_url: &'a str,
    workflow_id: &'a str,
    execution_path: &'a Path,
    config: &'a WorkflowConfig,
    input: Option<String>,
    task_queue_name: &'a str,
}

struct AuthParams<'a> {
    ca_cert_path: &'a str,
    client_cert_path: Option<&'a str>,
    client_key_path: Option<&'a str>,
    api_key: Option<&'a str>,
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

    let get_env_var = |name: &str| std::env::var(name).unwrap_or_else(|_| "".to_string());
    let ca_cert_path = get_env_var("MOOSE_TEMPORAL_CONFIG__CA_CERT");
    let client_cert_path = get_env_var("MOOSE_TEMPORAL_CONFIG__CLIENT_CERT");
    let client_key_path = get_env_var("MOOSE_TEMPORAL_CONFIG__CLIENT_KEY");
    let api_key = get_env_var("MOOSE_TEMPORAL_CONFIG__API_KEY");

    if is_local {
        execute_local_workflow(params).await
    } else if !ca_cert_path.is_empty()
        && !client_cert_path.is_empty()
        && !client_key_path.is_empty()
    {
        let auth_params = AuthParams {
            ca_cert_path: &ca_cert_path,
            client_cert_path: Some(&client_cert_path),
            client_key_path: Some(&client_key_path),
            api_key: None,
        };
        execute_cert_workflow(params, auth_params).await
    } else if !ca_cert_path.is_empty() && !api_key.is_empty() {
        let auth_params = AuthParams {
            ca_cert_path: &ca_cert_path,
            client_cert_path: None,
            client_key_path: None,
            api_key: Some(&api_key),
        };
        execute_api_key_workflow(params, auth_params).await
    } else {
        Err(TemporalExecutionError::AuthenticationError(
            "No authentication credentials provided for Temporal.".to_string(),
        ))
    }
}

async fn execute_local_workflow(
    params: WorkflowExecutionParams<'_>,
) -> Result<String, TemporalExecutionError> {
    let namespace = DEFAULT_TEMPORTAL_NAMESPACE.to_string();
    info!("Using namespace: {}", namespace);

    let endpoint = tonic::transport::Channel::from_shared(params.temporal_url.to_string())
        .map_err(|e| TemporalExecutionError::ConfigError(e.to_string()))?;

    let mut client = WorkflowServiceClient::new(endpoint.connect().await?);

    start_workflow_execution_with_channel(
        &mut client,
        namespace,
        params.workflow_id,
        params.execution_path,
        params.config,
        params.input,
        params.task_queue_name,
    )
    .await
}

async fn execute_cert_workflow(
    params: WorkflowExecutionParams<'_>,
    auth: AuthParams<'_>,
) -> Result<String, TemporalExecutionError> {
    let domain_name = params
        .temporal_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split(':')
        .next()
        .unwrap_or("");

    let namespace = domain_name
        .strip_suffix(".tmprl.cloud")
        .unwrap_or(domain_name)
        .to_string();

    info!("Using namespace: {}", namespace);
    info!("Using domain name for TLS: {}", domain_name);

    let client_cert_path = auth.client_cert_path.ok_or_else(|| {
        TemporalExecutionError::AuthenticationError(
            "Client certificate path is required".to_string(),
        )
    })?;
    let client_key_path = auth.client_key_path.ok_or_else(|| {
        TemporalExecutionError::AuthenticationError("Client key path is required".to_string())
    })?;

    let client_identity = tonic::transport::Identity::from_pem(
        std::fs::read(client_cert_path)
            .map_err(|e| TemporalExecutionError::AuthenticationError(e.to_string()))?,
        std::fs::read(client_key_path)
            .map_err(|e| TemporalExecutionError::AuthenticationError(e.to_string()))?,
    );

    let ca_certificate = tonic::transport::Certificate::from_pem(
        std::fs::read(auth.ca_cert_path)
            .map_err(|e| TemporalExecutionError::AuthenticationError(e.to_string()))?,
    );

    let tls_config = tonic::transport::ClientTlsConfig::new()
        .identity(client_identity)
        .ca_certificate(ca_certificate)
        .domain_name(domain_name);

    let endpoint = tonic::transport::Channel::from_shared(params.temporal_url.to_string())
        .map_err(|e| TemporalExecutionError::ConfigError(e.to_string()))?;

    let mut client = WorkflowServiceClient::new(endpoint.tls_config(tls_config)?.connect().await?);

    start_workflow_execution_with_channel(
        &mut client,
        namespace,
        params.workflow_id,
        params.execution_path,
        params.config,
        params.input,
        params.task_queue_name,
    )
    .await
}

async fn execute_api_key_workflow(
    params: WorkflowExecutionParams<'_>,
    auth: AuthParams<'_>,
) -> Result<String, TemporalExecutionError> {
    let domain_name = params
        .temporal_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split(':')
        .next()
        .unwrap_or("");

    let namespace = domain_name
        .strip_suffix(".tmprl.cloud")
        .unwrap_or(domain_name)
        .to_string();

    info!("Using namespace: {}", namespace);

    let ca_certificate = tonic::transport::Certificate::from_pem(
        std::fs::read(auth.ca_cert_path)
            .map_err(|e| TemporalExecutionError::AuthenticationError(e.to_string()))?,
    );

    let api_key = auth.api_key.ok_or_else(|| {
        TemporalExecutionError::AuthenticationError("API key is required".to_string())
    })?;

    let endpoint = tonic::transport::Channel::from_shared(
        "https://us-west1.gcp.api.temporal.io:7233".to_string(),
    )
    .map_err(|e| TemporalExecutionError::ConfigError(e.to_string()))?
    .tls_config(
        tonic::transport::ClientTlsConfig::new()
            .domain_name("us-west1.gcp.api.temporal.io")
            .ca_certificate(ca_certificate),
    )
    .map_err(|e| TemporalExecutionError::AuthenticationError(e.to_string()))?;

    struct ApiKeyInterceptor {
        api_key: String,
        namespace: String,
    }

    impl tonic::service::Interceptor for ApiKeyInterceptor {
        fn call(
            &mut self,
            mut request: tonic::Request<()>,
        ) -> Result<tonic::Request<()>, tonic::Status> {
            request.metadata_mut().insert(
                "authorization",
                tonic::metadata::MetadataValue::try_from(format!("Bearer {}", self.api_key))
                    .map_err(|_| tonic::Status::internal("Invalid metadata value"))?,
            );
            request.metadata_mut().insert(
                "temporal-namespace",
                tonic::metadata::MetadataValue::try_from(&self.namespace)
                    .map_err(|_| tonic::Status::internal("Invalid metadata value"))?,
            );
            Ok(request)
        }
    }

    let interceptor = ApiKeyInterceptor {
        api_key: api_key.to_string(),
        namespace: namespace.clone(),
    };

    let mut client = WorkflowServiceClient::with_interceptor(endpoint.connect_lazy(), interceptor);

    start_workflow_execution_with_interceptor(
        &mut client,
        namespace,
        params.workflow_id,
        params.execution_path,
        params.config,
        params.input,
        params.task_queue_name,
    )
    .await
}

async fn start_workflow_execution_with_channel(
    client: &mut WorkflowServiceClient<tonic::transport::Channel>,
    namespace: String,
    workflow_id: &str,
    execution_path: &Path,
    config: &WorkflowConfig,
    input: Option<String>,
    task_queue_name: &str,
) -> Result<String, TemporalExecutionError> {
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

    let request = tonic::Request::new(StartWorkflowExecutionRequest {
        namespace,
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

async fn start_workflow_execution_with_interceptor<I>(
    client: &mut WorkflowServiceClient<
        tonic::service::interceptor::InterceptedService<tonic::transport::Channel, I>,
    >,
    namespace: String,
    workflow_id: &str,
    execution_path: &Path,
    config: &WorkflowConfig,
    input: Option<String>,
    task_queue_name: &str,
) -> Result<String, TemporalExecutionError>
where
    I: tonic::service::Interceptor,
{
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

    let request = tonic::Request::new(StartWorkflowExecutionRequest {
        namespace,
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
