use anyhow::Result;
use std::sync::Arc;
use temporal_sdk_core::protos::coresdk::activity_task::ActivityTask;
use temporal_sdk_core::protos::coresdk::workflow_activation::WfActivation;
use temporal_sdk_core::protos::coresdk::workflow_completion::wf_activation_completion::Status;
use temporal_sdk_core::protos::coresdk::workflow_completion::{Success, WfActivationCompletion};
use temporal_sdk_core::protos::temporal::api::enums::v1::WorkflowExecutionStatus;
use temporal_sdk_core::{init, Core, CoreInitOptions, ServerGatewayOptions};
use url::Url;

#[derive(Clone)]
pub struct ScriptsOrchestrator {
    core: Arc<dyn Core>,
}

impl ScriptsOrchestrator {
    pub async fn new() -> Result<Self> {
        // Initialize Temporal core with default local settings
        let core = init(CoreInitOptions {
            gateway_opts: ServerGatewayOptions {
                target_url: Url::parse("http://localhost:7233").unwrap(),
                namespace: "default".to_string(),
                identity: "moose-scripts".to_string(),
                task_queue: "scripts-queue".to_string(),
                worker_binary_id: "moose-worker-1".to_string(),
                long_poll_timeout: std::time::Duration::from_secs(60),
            },
            evict_after_pending_cleared: true,
            max_outstanding_workflow_tasks: 10,
            max_outstanding_activities: 10,
        })
        .await?;

        Ok(Self {
            core: Arc::new(core),
        })
    }

    /// Start a new workflow execution
    pub async fn start_workflow(&self, workflow_id: String) -> Result<()> {
        let gateway = self.core.server_gateway();

        let response = gateway
            .start_workflow(
                "default".to_string(),
                "scripts-queue".to_string(),
                workflow_id,
                "test-workflow".to_string(),
            )
            .await?;

        println!("Started workflow: {:?}", response);
        Ok(())
    }

    /// Poll for workflow tasks and execute them
    pub async fn run_worker(&self) -> Result<()> {
        println!("Starting worker...");

        loop {
            // Poll for workflow tasks
            match self.core.poll_workflow_task().await {
                Ok(activation) => {
                    println!("Received workflow activation: {:?}", activation);

                    // For now just complete the workflow
                    let completion = WfActivationCompletion {
                        task_token: activation.task_token,
                        status: Some(Status::Successful(Success { commands: vec![] })),
                    };

                    if let Err(e) = self.core.complete_workflow_task(completion).await {
                        println!("Error completing workflow task: {:?}", e);
                    }
                }
                Err(e) => {
                    println!("Error polling workflow task: {:?}", e);
                }
            }
        }
    }

    /// Get the current status of a workflow
    pub async fn get_workflow_status(&self, _workflow_id: &str) -> Result<WorkflowExecutionStatus> {
        let wf_activation: WfActivation = self.core.poll_workflow_task().await?;

        let activity_activation: ActivityTask = self.core.poll_activity_task().await?;

        println!("wf_activation: {:?}", wf_activation);
        println!("activity_activation: {:?}", activity_activation);

        unimplemented!()

        // if let Some(last_event) = wf_activation.last() {
        //     match EventType::from_i32(last_event.event_type) {
        //         Some(EventType::WorkflowExecutionCompleted) => {
        //             Ok(WorkflowExecutionStatus::Completed)
        //         }
        //         Some(EventType::WorkflowExecutionFailed) => {
        //             Ok(WorkflowExecutionStatus::Failed)c
        //         }
        //         Some(EventType::WorkflowExecutionTimedOut) => {
        //             Ok(WorkflowExecutionStatus::TimedOut)
        //         }
        //         Some(EventType::WorkflowExecutionCanceled) => {
        //             Ok(WorkflowExecutionStatus::Canceled)
        //         }
        //         Some(EventType::WorkflowExecutionTerminated) => {
        //             Ok(WorkflowExecutionStatus::Terminated)
        //         }
        //         Some(EventType::WorkflowExecutionContinuedAsNew) => {
        //             Ok(WorkflowExecutionStatus::ContinuedAsNew)
        //         }
        //         _ => Ok(WorkflowExecutionStatus::Running)
        //     }
        // } else {
        //     Ok(WorkflowExecutionStatus::Running)
        // }
    }

    /// Wait for a workflow to complete with timeout
    pub async fn wait_for_workflow_completion(
        &self,
        workflow_id: &str,
        timeout: std::time::Duration,
    ) -> Result<WorkflowExecutionStatus> {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            let status = self.get_workflow_status(workflow_id).await?;

            match status {
                WorkflowExecutionStatus::Running => {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
                final_status => return Ok(final_status),
            }
        }

        Err(anyhow::anyhow!("Workflow did not complete within timeout"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio;

    #[tokio::test]
    async fn test_workflow_execution() -> Result<()> {
        let orchestrator = ScriptsOrchestrator::new().await?;

        // Start a test workflow
        let status = orchestrator.get_workflow_status("test-workflow-1").await?;
        println!("workflow_status: {:?}", status);

        Ok(())
    }

    #[tokio::test]
    async fn test_workflow_history() -> Result<()> {
        let orchestrator = ScriptsOrchestrator::new().await?;

        println!("test_workflow_history");

        let workflow_id = "test-workflow-history".to_string();
        let status = orchestrator.get_workflow_status(&workflow_id).await?;
        println!("workflow_status: {:?}", status);

        Ok(())
    }

    #[tokio::test]
    async fn test_workflow_lifecycle() -> Result<()> {
        let orchestrator = ScriptsOrchestrator::new().await?;
        let workflow_id = "test-workflow-lifecycle".to_string();

        // Start the workflow
        orchestrator.start_workflow(workflow_id.clone()).await?;

        // Check initial status
        let initial_status = orchestrator.get_workflow_status(&workflow_id).await?;
        assert_eq!(initial_status, WorkflowExecutionStatus::Running);

        // Start worker in background
        let orchestrator_clone = orchestrator.clone();
        let worker_handle = tokio::spawn(async move { orchestrator_clone.run_worker().await });

        // Wait for completion
        let final_status = orchestrator
            .wait_for_workflow_completion(&workflow_id, Duration::from_secs(30))
            .await?;

        assert_eq!(final_status, WorkflowExecutionStatus::Completed);

        // Clean up worker
        worker_handle.abort();

        Ok(())
    }
}
