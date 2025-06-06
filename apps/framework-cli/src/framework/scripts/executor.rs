use anyhow::Result;
use log::info;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::{config::WorkflowConfig, Workflow};
use crate::framework::{
    languages::SupportedLanguages,
    scripts::utils::{
        get_temporal_namespace, parse_schedule, parse_timeout_to_seconds, TemporalExecutionError,
    },
    scripts::Workflows,
};
use crate::infrastructure::orchestration::temporal::TemporalConfig;
use crate::infrastructure::orchestration::temporal_client::TemporalClientManager;
use crate::project::Project;
use crate::utilities::constants::{
    MOOSE_CLI_IDENTITY, PYTHON_TASK_QUEUE, TYPESCRIPT_TASK_QUEUE, WORKFLOW_TYPE,
};
use temporal_sdk_core::protos::temporal::api::common::v1::{
    Payload, Payloads, RetryPolicy, WorkflowType,
};
use temporal_sdk_core::protos::temporal::api::enums::v1::{
    TaskQueueKind, WorkflowIdConflictPolicy, WorkflowIdReusePolicy,
};

use temporal_sdk_core::protos::temporal::api::taskqueue::v1::TaskQueue;
use temporal_sdk_core::protos::temporal::api::workflowservice::v1::{
    ListWorkflowExecutionsRequest, StartWorkflowExecutionRequest,
};

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
    config: &WorkflowConfig,
    execution_path: &Path,
    input: Option<String>,
) -> Result<String, WorkflowExecutionError> {
    match language {
        SupportedLanguages::Python => {
            let params = WorkflowExecutionParams {
                temporal_config,
                workflow_id,
                execution_path,
                config,
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
                config,
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
    let namespace = get_temporal_namespace(&temporal_url);

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

/// Automatically starts all workflows that have a schedule configured.
///
/// # Assumptions:
/// - Can only start workflows that don't require input parameters, as there's no way
///   to determine what the input parameters should be at startup time.
/// - Workflows must have a non-empty schedule string in their config.toml
///
/// # Arguments
/// * `project` - The project configuration containing workflow settings and paths
///
/// # Returns
/// * `Result<(), WorkflowExecutionError>` - Success or an error if workflow startup fails
pub(crate) async fn execute_scheduled_workflows(
    project: &Project,
    dmv2_workflows: &HashMap<String, Workflow>,
) {
    if !project.features.workflows {
        info!("Workflows are not enabled for this project. Not auto-starting scheduled workflows");
        return;
    }

    let workflows = match Workflows::from_dir(project.scripts_dir()) {
        Ok(w) => w,
        Err(e) => {
            log::error!("Failed to read workflows during auto-start: {}", e);
            return;
        }
    };

    info!(
        "Auto-start workflows found {} workflows",
        workflows.get_defined_workflows().len() + dmv2_workflows.len()
    );

    let running_workflows = list_running_workflows(project).await;
    info!("Found {} running workflow IDs", running_workflows.len());

    async fn handle_workflow(
        workflow: &Workflow,
        running_workflows: &HashSet<String>,
        project: &Project,
    ) {
        if workflow.config.schedule.is_empty() {
            return;
        }

        if running_workflows.contains(&workflow.name) {
            info!(
                "Workflow {} is already running. Skipping auto-start",
                workflow.name
            );
            return;
        }

        match workflow.start(&project.temporal_config, None).await {
            Ok(_) => info!("Auto-started workflow: {}", workflow.name),
            Err(e) => log::error!("Failed to auto-start workflow {}: {}", workflow.name, e),
        }
    }

    // Handle regular workflows
    for workflow in workflows.get_defined_workflows() {
        handle_workflow(workflow, &running_workflows, project).await;
    }

    // Handle dmv2 workflows
    for workflow in dmv2_workflows.values() {
        handle_workflow(workflow, &running_workflows, project).await;
    }
}

async fn list_running_workflows(project: &Project) -> HashSet<String> {
    let client_manager = TemporalClientManager::new(&project.temporal_config);
    let temporal_url = project.temporal_config.temporal_url_with_scheme();
    let namespace = get_temporal_namespace(&temporal_url);

    let request = ListWorkflowExecutionsRequest {
        namespace: namespace.clone(),
        page_size: 1000,
        query: "ExecutionStatus = 'Running'".to_string(),
        ..Default::default()
    };

    match client_manager
        .execute(|mut client| async move {
            client
                .list_workflow_executions(request)
                .await
                .map_err(|e| anyhow::Error::msg(e.to_string()))
        })
        .await
    {
        Ok(response) => response
            .into_inner()
            .executions
            .into_iter()
            .filter_map(|execution| execution.execution)
            .map(|execution_info| execution_info.workflow_id)
            .collect(),
        Err(e) => {
            log::error!("Failed to list running workflows: {}", e);
            HashSet::new()
        }
    }
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

    let workflow_id = if let Some(data) = &params.input {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!(
            "{}-{:.16}",
            params.workflow_id,
            hex::encode(hasher.finalize())
        )
    } else {
        params.workflow_id.to_string()
    };

    Ok(StartWorkflowExecutionRequest {
        namespace,
        workflow_id,
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
        // Allow duplicate doesn't actually allow concurrent runs of the same workflow ID
        // It allows reuse of that workflow ID after the previous run has completed
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
