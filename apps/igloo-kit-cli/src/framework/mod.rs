use std::path::{PathBuf, Path};

use clap::Subcommand;
use reqwest::Url;

mod ingestion;
mod flow;
mod metric;
mod dataframe;
mod dashboard;
mod model;

enum Languages {
    Python,
    Typescript,
}

struct Template {
    name: String,
    language: Languages,
    path: PathBuf, // The path to the local file
    remote_path: Url, // The path to the remote file
}

trait FrameworkObject {
    fn new() -> Self;
    fn directory(&self) -> Path;
    fn templates(&self) -> Vec<Template>;
}

#[derive(Debug, Subcommand)]
pub enum AddableObjects {
    IngestPoint,
    Flow,
    Dataframe,
    Metric,
    Dashboard,
    Model,
}

