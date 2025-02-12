#[derive(Debug, thiserror::Error)]
pub enum TemporalExecutionError {
    #[error("Temportal connection error: {0}")]
    TemporalConnectionError(#[from] tonic::transport::Error),

    #[error("Temportal client error: {0}")]
    TemporalClientError(#[from] tonic::Status),

    #[error("Timeout error: {0}")]
    TimeoutError(String),
}
