#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BlocksError {
    #[error("Failed to start/stop the blocks process")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Default)]
pub struct Blocks {}

impl Blocks {
    pub fn id(&self) -> String {
        "onlyone".to_string()
    }
}
