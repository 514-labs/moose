#[derive(Debug, thiserror::Error)]
pub enum RedpandaChangesError {
    #[error("Not Supported - {0}")]
    NotSupported(String),

    #[error("Anyhow Error")]
    Other(#[from] anyhow::Error),
}
