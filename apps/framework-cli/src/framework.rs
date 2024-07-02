use clap::Subcommand;

pub mod aggregations;
pub mod consumption;
pub mod controller;
pub mod core;
pub mod data_model;
pub mod flows;
pub mod languages;
pub mod prisma;
pub mod python;
pub mod sdk;
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
