use hex::encode;
use sha2::{digest::Output, Sha256};
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct EndpointFile {
    pub path: PathBuf,
    pub hash: Output<Sha256>,
}

impl EndpointFile {
    pub fn id(&self) -> String {
        format!("{}-{}", self.path.to_string_lossy(), encode(self.hash))
    }
}

#[derive(Debug, Clone, Default)]
pub struct Consumption {
    pub endpoint_files: Vec<EndpointFile>,
}
