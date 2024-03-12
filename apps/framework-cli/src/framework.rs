use clap::Subcommand;

pub mod controller;
pub mod languages;
pub mod readme;
pub mod schema;
pub mod sdks;
pub mod transform;
pub mod typescript;
pub enum Insights {
    Metric,
    Dashboard,
    Model,
}

pub enum TopLevelObjects {
    Ingestion,
    Flow,
    DataModel,
    Insights(Insights),
}

#[derive(Debug, Subcommand)]
pub enum AddableObjects {
    IngestPoint,
    Flow,
    DataModel,
    Metric,
    Dashboard,
    Model,
}
