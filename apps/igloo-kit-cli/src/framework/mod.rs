use std::path::PathBuf;

use clap::Subcommand;
use reqwest::Url;

mod ingestion;
mod flow;
mod metric;
mod dataframe;
mod dashboard;
mod model;
pub mod directories;

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

