use std::path::{Path, PathBuf};

use super::FrameworkObject;

struct Ingestion;

impl FrameworkObject for Ingestion {
    fn new() -> Self {
        Ingestion
    }

    fn directory(&self) -> PathBuf {
        Path::new("ingestion").to_path_buf()
    }

    fn templates(&self) -> Vec<super::Template> {
        todo!()
    }
}