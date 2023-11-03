//! # CLI Commands
//! A module for all the commands that can be run from the CLI

use clap::{Args,  Subcommand};

use crate::framework::{self, languages::SupportedLanguages};


#[derive(Subcommand)]
pub enum Commands {
    // Initializes the developer environment with all the necessary directories including temporary ones for data storage
    /// Initialize your data-intensive app or service
    Init {
        /// Name of your app or service
        name: String,

        /// Language of your app or service
        #[arg(default_value_t = SupportedLanguages::Typescript, value_enum)]
        language: SupportedLanguages,

        /// Location of your app or service
        #[arg(default_value = ".")]
        location: String,
    },
    // Adds a new templated object to the project
    Add(AddArgs),
    /// Starts a local development environment to build your data-intensive app or service
    Dev{},
    // Updates the redpanda cluster and clickhouse database with the latest objects
    Update{},
    // Stops development infrastructure
    Stop{},
    // Clears all temporary data and stops development infrastructure
    Clean{},
}

#[derive(Debug, Args)]
#[command()]
pub struct AddArgs {
    #[command(subcommand)]
    pub command: Option<framework::AddableObjects>,
}


