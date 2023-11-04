//! # CLI Commands
//! A module for all the commands that can be run from the CLI

use clap::{Args, Subcommand};

use crate::framework;

#[derive(Subcommand)]
pub enum Commands {
    // Initializes the developer environment with all the necessary directories including temporary ones for data storage
    Init {},
    // Adds a new templated object to the project
    Add(AddArgs),
    // Spins up development infrastructure including a redpanda cluster and clickhouse database
    Dev {},
    // Updates the redpanda cluster and clickhouse database with the latest objects
    Update {},
    // Stops development infrastructure
    Stop {},
    // Clears all temporary data and stops development infrastructure
    Clean {},
}

#[derive(Debug, Args)]
#[command()]
pub struct AddArgs {
    #[command(subcommand)]
    pub command: Option<framework::AddableObjects>,
}
