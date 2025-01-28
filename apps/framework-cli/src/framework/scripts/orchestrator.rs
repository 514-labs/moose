use std::{sync::Arc, time::Duration};

use temporal_sdk_core::{
    init,
    protos::temporal::api::{
        common::v1::Payloads,
        enums::v1::EventType,
        workflowservice::v1::{
            workflow_service_client::WorkflowServiceClient, DescribeWorkflowExecutionRequest,
            GetWorkflowExecutionHistoryRequest, ResetWorkflowExecutionRequest,
            ResetWorkflowExecutionResponse, SignalWorkflowExecutionResponse,
        },
    },
    Core, CoreInitOptions, ServerGatewayApis, ServerGatewayOptions,
};

use temporal_sdk_core::protos::temporal::api::enums::v1::WorkflowExecutionStatus;
use tonic::Status;

/// This is not currently used but could be useful in the future.
#[allow(dead_code)]
async fn get_core_gateway() -> Arc<dyn ServerGatewayApis> {
    init(CoreInitOptions {
        gateway_opts: ServerGatewayOptions {
            target_url: "https://localhost:7233".parse().unwrap(),
            namespace: "default".to_string(),
            task_queue: "default".to_string(),
            identity: "default".to_string(),
            worker_binary_id: "default".to_string(),
            long_poll_timeout: Duration::from_secs(10),
        },
        evict_after_pending_cleared: true,
        max_outstanding_workflow_tasks: 100,
        max_outstanding_activities: 100,
    })
    .await
    .unwrap()
    .server_gateway()
}

/// This is not currently used but could be useful in the future.
#[allow(dead_code)]
async fn signal_workflow(
    gateway: &(dyn ServerGatewayApis),
    workflow_id: impl Into<String>,
    signal_name: impl Into<String>,
    payloads: Option<Payloads>,
) -> Result<SignalWorkflowExecutionResponse, Status> {
    gateway
        .signal_workflow_execution(
            workflow_id.into(),
            "".to_string(),
            signal_name.into(),
            payloads,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
}

/// Reset a workflow. This ends the current workflow and starts a new one
/// from the last known good state.
///
/// This is not currently used but could be useful in the future.
#[allow(dead_code)]
async fn reset_workflow(
    workflow_id: impl Into<String>,
) -> Result<ResetWorkflowExecutionResponse, Status> {
    let workflow_id = workflow_id.into();
    let endpoint = tonic::transport::Endpoint::from_static("http://localhost:7233");

    let mut client = WorkflowServiceClient::connect(endpoint)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    // First get the workflow history
    let history_request = tonic::Request::new(GetWorkflowExecutionHistoryRequest {
        namespace: "default".to_string(),
        execution: Some(
            temporal_sdk_core::protos::temporal::api::common::v1::WorkflowExecution {
                workflow_id: workflow_id.clone(),
                run_id: "".to_string(),
            },
        ),
        maximum_page_size: 1000,
        next_page_token: vec![],
        wait_new_event: false,
        history_event_filter_type: 0,
        skip_archival: false,
    });

    let history_response = client
        .get_workflow_execution_history(history_request)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    // Find the last workflow task completed event
    let workflow_task_completed_event_id = history_response
        .into_inner()
        .history
        .ok_or_else(|| Status::internal("No history found"))?
        .events
        .into_iter()
        .filter(|event| event.event_type == EventType::WorkflowTaskCompleted as i32)
        .map(|event| event.event_id)
        .last()
        .ok_or_else(|| Status::internal("No workflow task completed event found"))?;

    println!(
        "Workflow task completed event ID: {}",
        workflow_task_completed_event_id
    );

    let request = tonic::Request::new(ResetWorkflowExecutionRequest {
        namespace: "default".to_string(),
        workflow_execution: Some(
            temporal_sdk_core::protos::temporal::api::common::v1::WorkflowExecution {
                workflow_id: workflow_id.clone(),
                run_id: "".to_string(),
            },
        ),
        reason: "Resetting failed workflow".to_string(),
        workflow_task_finish_event_id: workflow_task_completed_event_id,
        request_id: uuid::Uuid::new_v4().to_string(),
    });

    client
        .reset_workflow_execution(request)
        .await
        .map(|response| response.into_inner())
        .map_err(|e| Status::internal(e.to_string()))
}

/// Check if a workflow is currently running
pub async fn is_workflow_running(workflow_id: impl Into<String>) -> Result<bool, Status> {
    let workflow_id = workflow_id.into();
    let endpoint = tonic::transport::Endpoint::from_static("http://localhost:7233");
    let mut client = WorkflowServiceClient::connect(endpoint)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let request = tonic::Request::new(DescribeWorkflowExecutionRequest {
        namespace: "default".to_string(),
        execution: Some(
            temporal_sdk_core::protos::temporal::api::common::v1::WorkflowExecution {
                workflow_id,
                run_id: "".to_string(),
            },
        ),
    });

    let response = client
        .describe_workflow_execution(request)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    println!("Response: {:?}", response);

    let status = response
        .into_inner()
        .workflow_execution_info
        .ok_or_else(|| Status::internal("No workflow execution info found"))?
        .status;

    println!("Workflow status: {}", status);

    // Only consider status 1 (Running) as active
    Ok(status == WorkflowExecutionStatus::Running as i32)
}

/// Stop a workflow
pub async fn stop_workflow(workflow_id: impl Into<String>) -> Result<(), Status> {
    let workflow_id = workflow_id.into();
    let endpoint = tonic::transport::Endpoint::from_static("http://localhost:7233");
    let mut client = WorkflowServiceClient::connect(endpoint)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    let request = tonic::Request::new(
        temporal_sdk_core::protos::temporal::api::workflowservice::v1::TerminateWorkflowExecutionRequest {
            namespace: "default".to_string(),
            workflow_execution: Some(
                temporal_sdk_core::protos::temporal::api::common::v1::WorkflowExecution {
                    workflow_id,
                    run_id: "".to_string(),
                },
            ),
            reason: "Workflow terminated by user".to_string(),
            details: None,
            identity: "default".to_string(),
            first_execution_run_id: "".to_string(),
        }
    );

    client
        .terminate_workflow_execution(request)
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    // use std::fs;
    // use std::path::PathBuf;
    // use tempfile::tempdir;

    //     fn create_failing_workflow() -> Result<(tempfile::TempDir, PathBuf, String)> {
    //         let temp_dir = tempdir()?;
    //         let workflow_id = "failing-workflow";
    //         let workflow_dir = temp_dir.path().join(workflow_id);
    //         fs::create_dir_all(&workflow_dir)?;

    //         // 1. Extract script (works)
    //         fs::write(
    //             workflow_dir.join("1.extract.py"),
    //             r#"
    // from moose_lib import task

    // @task
    // def extract():
    //     return {"step": "extract", "data": [1, 2, 3]}
    // "#,
    //         )?;

    //         // 2. Transform script (fails)
    //         fs::write(
    //             workflow_dir.join("2.failing.py"),
    //             r#"
    // from moose_lib import task

    // @task
    // def failing():
    //     raise ValueError("Intentional user error!")
    // "#,
    //         )?;

    //         // 3. Load script (works)
    //         fs::write(
    //             workflow_dir.join("3.load.py"),
    //             r#"
    // from moose_lib import task

    // @task
    // def load():
    //     return {"step": "load", "msg": "Loaded data successfully"}
    // "#,
    //         )?;

    //         Ok((temp_dir, workflow_dir, workflow_id.to_string()))
    //     }

    //     fn unique_workflow_id() -> String {
    //         format!("test-workflow-{}", uuid::Uuid::new_v4())
    //     }

    //     fn create_unique_workflow() -> Result<(tempfile::TempDir, PathBuf, String)> {
    //         let workflow_id = unique_workflow_id();
    //         let temp_dir = tempdir()?;
    //         let workflow_dir = temp_dir.path().join(workflow_id.clone());
    //         fs::create_dir_all(&workflow_dir)?;

    //         // 1. Extract script (works) with a sleep
    //         fs::write(
    //             workflow_dir.join("1.extract.py"),
    //             r#"
    // from moose_lib import task
    // import time

    // @task
    // def extract():
    //     time.sleep(1)
    //     return {"step": "extract", "data": [1, 2, 3]}
    // "#,
    //         )?;

    //         // 2. Transform script (fails)
    //         fs::write(
    //             workflow_dir.join("2.transform.py"),
    //             r#"
    // from moose_lib import task
    // import time
    // @task
    // def transform():
    //     time.sleep(1)
    //     return {"step": "extract", "data": [1, 2, 3]}
    // "#,
    //         )?;

    //         // 3. Load script (works)
    //         fs::write(
    //             workflow_dir.join("3.load.py"),
    //             r#"
    // from moose_lib import task
    // import time

    // @task
    // def load():
    //     time.sleep(1)
    //     return {"step": "load", "msg": "Loaded data successfully"}
    // "#,
    //         )?;

    //         let id = workflow_id.clone();
    //         Ok((temp_dir, workflow_dir, id))
    //     }

    #[ignore]
    #[tokio::test]
    async fn test_reset_workflow() -> Result<(), Status> {
        // Create failing workflow
        // let (temp_dir, workflow_dir) =
        //     create_failing_workflow().map_err(|e| Status::internal(e.to_string()))?;

        // First run should fail
        // let result = execute_workflow(SupportedLanguages::Python, &workflow_dir, None).await;
        // assert!(result.is_err(), "Expected workflow to fail");

        // Reset the workflow
        let response = reset_workflow("failing-workflow").await?;
        assert!(response.run_id.len() > 0, "Expected new run ID");

        // Keep temp_dir alive until end of test
        // drop(temp_dir);
        Ok(())
    }
}
