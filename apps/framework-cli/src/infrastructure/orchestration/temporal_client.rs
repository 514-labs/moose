use anyhow::{Error, Result};
use lazy_static::lazy_static;
use log::info;
use temporal_sdk_core_protos::temporal::api::workflowservice::v1::workflow_service_client::WorkflowServiceClient;
use temporal_sdk_core_protos::temporal::api::workflowservice::v1::{
    DescribeWorkflowExecutionRequest, DescribeWorkflowExecutionResponse,
    GetWorkflowExecutionHistoryRequest, GetWorkflowExecutionHistoryResponse,
    ListWorkflowExecutionsRequest, ListWorkflowExecutionsResponse, SignalWorkflowExecutionRequest,
    SignalWorkflowExecutionResponse, StartWorkflowExecutionRequest,
    TerminateWorkflowExecutionRequest, TerminateWorkflowExecutionResponse,
};
use tonic::service::interceptor::InterceptedService;
use tonic::transport::{Channel, Uri};

use crate::framework::scripts::utils::{get_temporal_domain_name, get_temporal_namespace};

pub struct TemporalClientManager {
    temporal_url: String,
}

pub enum TemporalClient {
    Standard(WorkflowServiceClient<Channel>),
    WithInterceptor(WorkflowServiceClient<InterceptedService<Channel, ApiKeyInterceptor>>),
}

pub struct ApiKeyInterceptor {
    api_key: String,
    namespace: String,
}

fn get_env_var(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| "".to_string())
}

lazy_static! {
    pub static ref MOOSE_TEMPORAL_CONFIG__CA_CERT: String =
        get_env_var("MOOSE_TEMPORAL_CONFIG__CA_CERT");
    pub static ref MOOSE_TEMPORAL_CONFIG__CLIENT_CERT: String =
        get_env_var("MOOSE_TEMPORAL_CONFIG__CLIENT_CERT");
    pub static ref MOOSE_TEMPORAL_CONFIG__CLIENT_KEY: String =
        get_env_var("MOOSE_TEMPORAL_CONFIG__CLIENT_KEY");
    pub static ref MOOSE_TEMPORAL_CONFIG__API_KEY: String =
        get_env_var("MOOSE_TEMPORAL_CONFIG__API_KEY");
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

impl TemporalClientManager {
    pub fn new(temporal_url: &str) -> Self {
        Self {
            temporal_url: temporal_url.to_string(),
        }
    }

    pub async fn execute<F, Fut, R>(&self, operation: F) -> Result<R>
    where
        F: FnOnce(TemporalClient) -> Fut,
        Fut: std::future::Future<Output = Result<R>>,
    {
        let client = self.get_client().await?;
        operation(client).await
    }

    async fn get_client(&self) -> Result<TemporalClient> {
        let is_local = self.temporal_url.contains("localhost");
        info!("Getting client for Temporal URL: {}", self.temporal_url);

        if is_local {
            let client = get_temporal_client(&self.temporal_url).await?;
            Ok(TemporalClient::Standard(client))
        } else if !MOOSE_TEMPORAL_CONFIG__CA_CERT.is_empty()
            && !MOOSE_TEMPORAL_CONFIG__CLIENT_CERT.is_empty()
            && !MOOSE_TEMPORAL_CONFIG__CLIENT_KEY.is_empty()
        {
            let client = get_temporal_client_mtls(&self.temporal_url).await?;
            Ok(TemporalClient::Standard(client))
        } else if !MOOSE_TEMPORAL_CONFIG__CA_CERT.is_empty()
            && !MOOSE_TEMPORAL_CONFIG__API_KEY.is_empty()
        {
            let client = get_temporal_client_api_key(&self.temporal_url).await?;
            Ok(TemporalClient::WithInterceptor(client))
        } else {
            Err(Error::msg(
                "No authentication credentials provided for Temporal.",
            ))
        }
    }
}

impl TemporalClient {
    pub async fn start_workflow_execution(
        &mut self,
        request: StartWorkflowExecutionRequest,
    ) -> Result<String> {
        match self {
            TemporalClient::Standard(client) => {
                let response = client
                    .start_workflow_execution(tonic::Request::new(request))
                    .await?;
                Ok(response.into_inner().run_id)
            }
            TemporalClient::WithInterceptor(client) => {
                let response = client
                    .start_workflow_execution(tonic::Request::new(request))
                    .await?;
                Ok(response.into_inner().run_id)
            }
        }
    }

    pub async fn list_workflow_executions(
        &mut self,
        request: ListWorkflowExecutionsRequest,
    ) -> Result<tonic::Response<ListWorkflowExecutionsResponse>> {
        match self {
            TemporalClient::Standard(client) => client
                .list_workflow_executions(request)
                .await
                .map_err(Error::from),
            TemporalClient::WithInterceptor(client) => client
                .list_workflow_executions(request)
                .await
                .map_err(Error::from),
        }
    }

    pub async fn signal_workflow_execution(
        &mut self,
        request: SignalWorkflowExecutionRequest,
    ) -> Result<tonic::Response<SignalWorkflowExecutionResponse>> {
        match self {
            TemporalClient::Standard(client) => client
                .signal_workflow_execution(request)
                .await
                .map_err(Error::from),
            TemporalClient::WithInterceptor(client) => client
                .signal_workflow_execution(request)
                .await
                .map_err(Error::from),
        }
    }

    pub async fn terminate_workflow_execution(
        &mut self,
        request: TerminateWorkflowExecutionRequest,
    ) -> Result<tonic::Response<TerminateWorkflowExecutionResponse>> {
        match self {
            TemporalClient::Standard(client) => client
                .terminate_workflow_execution(request)
                .await
                .map_err(Error::from),
            TemporalClient::WithInterceptor(client) => client
                .terminate_workflow_execution(request)
                .await
                .map_err(Error::from),
        }
    }

    pub async fn describe_workflow_execution(
        &mut self,
        request: DescribeWorkflowExecutionRequest,
    ) -> Result<tonic::Response<DescribeWorkflowExecutionResponse>> {
        match self {
            TemporalClient::Standard(client) => client
                .describe_workflow_execution(request)
                .await
                .map_err(Error::from),
            TemporalClient::WithInterceptor(client) => client
                .describe_workflow_execution(request)
                .await
                .map_err(Error::from),
        }
    }

    pub async fn get_workflow_execution_history(
        &mut self,
        request: GetWorkflowExecutionHistoryRequest,
    ) -> Result<tonic::Response<GetWorkflowExecutionHistoryResponse>> {
        match self {
            TemporalClient::Standard(client) => client
                .get_workflow_execution_history(request)
                .await
                .map_err(Error::from),
            TemporalClient::WithInterceptor(client) => client
                .get_workflow_execution_history(request)
                .await
                .map_err(Error::from),
        }
    }
}

async fn connect_to_temporal(temporal_url: &str) -> Result<WorkflowServiceClient<Channel>> {
    let endpoint: Uri = temporal_url.parse().unwrap();
    WorkflowServiceClient::connect(endpoint).await.map_err(|_| {
        Error::msg("Could not connect to Temporal. Please ensure the Temporal server is running.")
    })
}

pub async fn get_temporal_client(temporal_url: &str) -> Result<WorkflowServiceClient<Channel>> {
    connect_to_temporal(temporal_url).await.map_err(|e| {
        eprintln!("{}", e);
        Error::msg(format!("{}", e))
    })
}

pub async fn get_temporal_client_mtls(
    temporal_url: &str,
) -> Result<WorkflowServiceClient<Channel>> {
    let ca_cert_path = MOOSE_TEMPORAL_CONFIG__CA_CERT.clone();
    let client_cert_path = MOOSE_TEMPORAL_CONFIG__CLIENT_CERT.clone();
    let client_key_path = MOOSE_TEMPORAL_CONFIG__CLIENT_KEY.clone();

    let domain_name = get_temporal_domain_name(temporal_url);

    let client_identity = tonic::transport::Identity::from_pem(
        std::fs::read(client_cert_path).map_err(|e| Error::msg(e.to_string()))?,
        std::fs::read(client_key_path).map_err(|e| Error::msg(e.to_string()))?,
    );

    let ca_certificate = tonic::transport::Certificate::from_pem(
        std::fs::read(ca_cert_path).map_err(|e| Error::msg(e.to_string()))?,
    );

    let tls_config = tonic::transport::ClientTlsConfig::new()
        .identity(client_identity)
        .ca_certificate(ca_certificate)
        .domain_name(domain_name);

    let endpoint = tonic::transport::Channel::from_shared(temporal_url.to_string())
        .map_err(|e| Error::msg(e.to_string()))?;

    let client = WorkflowServiceClient::new(endpoint.tls_config(tls_config)?.connect().await?);

    Ok(client)
}

pub async fn get_temporal_client_api_key(
    temporal_url: &str,
) -> Result<WorkflowServiceClient<InterceptedService<Channel, ApiKeyInterceptor>>> {
    let ca_cert_path = MOOSE_TEMPORAL_CONFIG__CA_CERT.clone();
    let api_key = MOOSE_TEMPORAL_CONFIG__API_KEY.clone();

    let domain_name = get_temporal_domain_name(temporal_url);

    let namespace = get_temporal_namespace(domain_name);

    let ca_certificate = tonic::transport::Certificate::from_pem(
        std::fs::read(ca_cert_path).map_err(|e| Error::msg(e.to_string()))?,
    );

    let endpoint =
        tonic::transport::Channel::from_shared("https://us-west1.gcp.api.temporal.io:7233")
            .map_err(|e| Error::msg(e.to_string()))?
            .tls_config(
                tonic::transport::ClientTlsConfig::new()
                    .domain_name("us-west1.gcp.api.temporal.io")
                    .ca_certificate(ca_certificate),
            )
            .map_err(|e| Error::msg(e.to_string()))?;

    let interceptor = ApiKeyInterceptor {
        api_key: api_key.to_string(),
        namespace: namespace.clone(),
    };

    let client = WorkflowServiceClient::with_interceptor(endpoint.connect_lazy(), interceptor);

    Ok(client)
}
