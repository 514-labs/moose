use clap::Subcommand;

pub mod directories;
pub mod schema;
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

