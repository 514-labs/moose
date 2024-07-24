use clap::Subcommand;

pub mod aggregations;
pub mod bulk_import;
pub mod consumption;
pub mod controller;
pub mod core;
pub mod data_model;
pub mod languages;
pub mod prisma;
pub mod python;
pub mod sdk;
pub mod streaming;
pub mod typescript;

pub enum Insights {
    Metric,
    Dashboard,
    Model,
}

pub enum TopLevelObjects {
    Ingestion,
    StreamingFunction,
    DataModel,
    Insights(Insights),
}

#[derive(Debug, Subcommand)]
pub enum AddableObjects {
    IngestPoint,
    StreamingFunction,
    DataModel,
    Metric,
    Dashboard,
    Model,
}
