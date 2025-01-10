use bytes::Bytes;
use http::Request;
use http_body::Body;
use std::error::Error as StdError;
use temporal_sdk_core::protos::temporal::api::{
    common::v1::{WorkflowExecution, WorkflowType},
    taskqueue::v1::TaskQueue,
    workflowservice::v1::{
        workflow_service_client::WorkflowServiceClient, DescribeWorkflowExecutionRequest,
        StartWorkflowExecutionRequest,
    },
};
use tonic::body::BoxBody;
use tonic::codegen::Service;
use tonic::transport::Channel;

#[derive(Clone)]
pub struct Workflow {
    pub namespace: String,
    pub id: String,
    pub _type: String,
    pub task_queue: String,
}

pub struct TemporalClient {
    client: WorkflowServiceClient<Channel>,
}

impl TemporalClient
where
    Channel: tonic::client::GrpcService<BoxBody>,
    Channel::Error: Into<StdError>,
    Channel::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <Channel::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub async fn start_workflow(&mut self, workflow: &Workflow) -> Result<String, tonic::Status>
    where
        Channel: Service<Request<BoxBody>> + Send + Sync + 'static,
    {
        let request = StartWorkflowExecutionRequest {
            namespace: workflow.namespace.clone(),
            workflow_id: workflow.id.clone(),
            workflow_type: Some(WorkflowType {
                name: workflow._type.clone(),
            }),
            task_queue: Some(TaskQueue {
                name: workflow.task_queue.clone(),
                ..Default::default()
            }),
            request_id: uuid::Uuid::new_v4().to_string(),
            ..Default::default()
        };

        let response = self.client.start_workflow_execution(request).await?;
        Ok(response.into_inner().run_id)
    }

    pub async fn describe_workflow(
        &mut self,
        namespace: String,
        workflow_id: String,
        run_id: Option<String>,
    ) -> Result<WorkflowExecution, tonic::Status> {
        let request = DescribeWorkflowExecutionRequest {
            namespace,
            execution: Some(WorkflowExecution {
                workflow_id,
                run_id: run_id.unwrap_or_default(),
            }),
        };

        let response = self.client.describe_workflow_execution(request).await?;
        Ok(response
            .into_inner()
            .workflow_execution_info
            .unwrap()
            .execution
            .unwrap())
    }
}
