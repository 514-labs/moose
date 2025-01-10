use anyhow::Result;
use std::sync::Arc;
use temporal_sdk_core::protos::coresdk::activity_task::ActivityTask;
use temporal_sdk_core::protos::coresdk::common::Payload;
use temporal_sdk_core::protos::coresdk::workflow_activation::{self, WfActivation};
use temporal_sdk_core::protos::coresdk::workflow_commands::{self, WorkflowCommand};
use temporal_sdk_core::protos::coresdk::workflow_completion::wf_activation_completion::Status;
use temporal_sdk_core::protos::coresdk::workflow_completion::{Success, WfActivationCompletion};
use temporal_sdk_core::protos::temporal::api::enums::v1::WorkflowExecutionStatus;

use temporal_sdk_core::{init, Core, CoreInitOptions, ServerGatewayOptions};
use url::Url;

use super::activity::{Activity, ScriptPayload};

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
    pub async fn run_worker(&self, activities: Vec<Activity>) -> Result<()> {
        println!("Starting worker...");

        loop {
            // Poll for workflow tasks
            match self.core.poll_workflow_task().await {
                Ok(activation) => {
                    println!("Received workflow activation: {:?}", activation);

                    // Execute the activities
                    self.handle_activation(activation, &activities).await?;
                }
                Err(e) => {
                    println!("Error polling workflow task: {:?}", e);
                }
            }

            // Poll for activity tasks
            match self.core.poll_activity_task().await {
                Ok(activity) => {
                    println!("Received activity task: {:?}", activity);
                }
                Err(e) => {
                    println!("Error polling activity task: {:?}", e);
                }
            }

            self.core.shutdown().await;
        }
    }

    pub async fn handle_activation(
        &self,
        activation: WfActivation,
        activities: &Vec<Activity>,
    ) -> Result<()> {
        // Create commands for each activity
        let commands: Vec<WorkflowCommand> = activities
            .iter()
            .enumerate()
            .map(|(_, activity)| WorkflowCommand {
                variant: Some(
                    workflow_commands::workflow_command::Variant::ScheduleActivity(
                        activity.schedule(ScriptPayload {
                            data: vec![], // Add your payload data here
                            metadata: Default::default(),
                        }),
                    ),
                ),
            })
            .collect();

        // Complete workflow activation with all activity commands
        let completion = WfActivationCompletion {
            task_token: activation.task_token,
            status: Some(Status::Successful(Success { commands })),
        };

        self.core.complete_workflow_task(completion).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio;

    #[tokio::test]
    async fn test_workflow_lifecycle() -> Result<()> {
        let orchestrator = ScriptsOrchestrator::new().await?;
        let workflow_id = "test-workflow-lifecycle".to_string();

        // Start the workflow
        orchestrator.start_workflow(workflow_id.clone()).await?;

        // Start worker in background
        let orchestrator_clone = orchestrator.clone();

        let activities = vec![Activity::builder()
            .id("python-script-1")
            ._type("execute_script")
            .namespace("default")
            .task_queue_name("script-queue")
            .build()];

        let worker_handle =
            tokio::spawn(async move { orchestrator_clone.run_worker(activities).await });

        // Clean up worker
        worker_handle.abort();

        Ok(())
    }
}
