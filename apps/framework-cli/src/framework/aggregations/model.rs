#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AggregationError {
    #[error("Failed to start/stop the aggregation process")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Default)]
pub struct Aggregation {}

impl Aggregation {
    pub fn id(&self) -> String {
        "onlyone".to_string()
    }
}
