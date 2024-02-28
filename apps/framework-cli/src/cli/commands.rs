//! # CLI Commands
//! A module for all the commands that can be run from the CLI

use clap::Subcommand;

use crate::framework::languages::SupportedLanguages;

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
    // Link {
    //     /// Name of your client application or service (ex. `my-blog`)
    //     name: String,

    //     /// Language of your app or service
    //     #[arg(default_value_t = SupportedLanguages::Typescript, value_enum)]
    //     language: SupportedLanguages,

    //     /// Location of your app or service
    //     #[arg(default_value = ".")]
    //     location: String,

    //     /// Name of the project to link to. Pulls the list of projects from the config file
    //     project: String,
    // },
    /// Starts a local development environment to build your data-intensive app or service
    Dev {},
    // Updates the redpanda cluster and clickhouse database with the latest objects
    Update {},
    // Stops development infrastructure
    Stop {},
    // Clears all temporary data and stops development infrastructure
    Clean {},
    /// Docker related commands
    Docker {
        sub_method: String,
    },
}
