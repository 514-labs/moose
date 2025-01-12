use temporal_sdk_core::{
    protos::temporal::api::{
        common::v1::Payloads, workflowservice::v1::SignalWorkflowExecutionResponse,
    },
    ServerGatewayApis,
};

use tonic::Status;

async fn signal_workflow(
    gateway: &impl ServerGatewayApis,
    workflow_id: String,
    signal_name: String,
    payloads: Option<Payloads>,
) -> Result<SignalWorkflowExecutionResponse, Status> {
    gateway
        .signal_workflow_execution(workflow_id, "".to_string(), signal_name, payloads)
        .await
        .map_err(|e| Status::internal(e.to_string()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert!(true);
    }
}
