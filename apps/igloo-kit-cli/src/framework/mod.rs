use std::path::PathBuf;

use clap::Subcommand;

pub mod directories;
pub mod schema;

enum Languages {
    Python,
    Typescript,
}

struct Template {
    name: String,
    language: Languages,
    path: PathBuf, // The path to the local file
    // remote_path: Url, // The path to the remote file
}

trait FrameworkObject {
    fn new() -> Self;
    fn directory(&self) -> PathBuf;
    fn templates(&self) -> Vec<Template>;
}

pub enum Insights {
    Metric,
    Dashboard,
    Model,
}

pub enum TopLevelObjects {
    Ingestion,
    Flow,
    Dataframe,
    Insights(Insights),
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

