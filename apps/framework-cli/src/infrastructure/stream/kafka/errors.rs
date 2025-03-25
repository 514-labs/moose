#[derive(Debug, thiserror::Error)]
pub enum KafkaChangesError {
    #[error("Not Supported - {0}")]
    NotSupported(String),

    #[error("Anyhow Error")]
    Other(#[from] anyhow::Error),
}
