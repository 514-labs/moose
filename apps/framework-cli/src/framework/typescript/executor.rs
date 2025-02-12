use std::collections::HashMap;
use std::path::Path;

use crate::framework::scripts::config::WorkflowConfig;
use crate::framework::scripts::errors::TemporalExecutionError;

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
const TYPESCRIPT_TASK_QUEUE: &str = "typescript-script-queue";
const MOOSE_CLI_IDENTITY: &str = "moose-cli";

fn parse_timeout_to_seconds(timeout: &str) -> Result<i64, TemporalExecutionError> {
    if timeout.is_empty() {
        return Err(TemporalExecutionError::TimeoutError(
            "Timeout string is empty".to_string(),
        ));
    }

    let (value, unit) = timeout.split_at(timeout.len() - 1);
    let value: u64 = value
        .parse()
        .map_err(|_| TemporalExecutionError::TimeoutError("Invalid number format".to_string()))?;

    let seconds =
        match unit {
            "h" => value * 3600,
            "m" => value * 60,
            "s" => value,
            _ => return Err(TemporalExecutionError::TimeoutError(
                "Invalid time unit. Must be h, m, or s for hours, minutes, or seconds respectively"
                    .to_string(),
            )),
        };

    Ok(seconds as i64)
}

fn parse_schedule(schedule: &str) -> String {
    if schedule.is_empty() {
        return String::new();
    }

    match schedule {
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
    }
}

pub(crate) async fn execute_typescript_workflow(
    workflow_id: &str,
    execution_path: &Path,
    config: &WorkflowConfig,
    input: Option<String>,
) -> Result<String, TemporalExecutionError> {
    // TODO: Make this configurable
    let endpoint = tonic::transport::Endpoint::from_static("http://localhost:7233");
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
            name: TYPESCRIPT_TASK_QUEUE.to_string(),
            kind: TaskQueueKind::Normal as i32,
            normal_name: TYPESCRIPT_TASK_QUEUE.to_string(),
        }),
        input: Some(Payloads { payloads }),
        workflow_execution_timeout: None,
        workflow_run_timeout: Some(prost_wkt_types::Duration {
            seconds: parse_timeout_to_seconds(&config.timeout)?,
            nanos: 0,
        }),
        workflow_task_timeout: None,
        identity: MOOSE_CLI_IDENTITY.to_string(),
        request_id: uuid::Uuid::new_v4().to_string(),
        search_attributes: None,
        header: None,
        workflow_id_reuse_policy: WorkflowIdReusePolicy::AllowDuplicate as i32,
        retry_policy: Some(RetryPolicy {
            // The number of retries a workflow run will be attempted
            maximum_attempts: config.retries as i32,
            ..Default::default()
        }),
        cron_schedule: parse_schedule(&config.schedule),
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

    let response = client.start_workflow_execution(request).await?;
    let run_id = response.into_inner().run_id;
    Ok(run_id)
}
