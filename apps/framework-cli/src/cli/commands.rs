//! # CLI Commands
//! A module for all the commands that can be run from the CLI

use clap::{Args, Subcommand};

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

        /// Template to use for the project
        #[arg(short, long)]
        template: Option<String>,
    },
    /// Builds your moose project
    Build {
        #[arg(short, long)]
        docker: bool,
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
    /// Start a remote environment for use in cloud deployments
    Prod {},
    /// Generates missing migration files
    Generate(GenerateArgs),
    BumpVersion {
        new_version: Option<String>,
    },
    // Updates the redpanda cluster and clickhouse database with the latest objects
    Update {},
    // Stops development infrastructure
    Stop {},
    // Clears all temporary data and stops development infrastructure
    Clean {},
    /// TODO: Add description for flow command
    Flow(FlowArgs),
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct FlowArgs {
    #[command(subcommand)]
    pub command: Option<FlowCommands>,
}

#[derive(Debug, Subcommand)]
pub enum FlowCommands {
    /// TODO: Add description for flow create sub-command
    #[command(arg_required_else_help = true)]
    Create(FlowCreateArgs),
}

#[derive(Debug, Args)]
pub struct FlowCreateArgs {
    #[arg(short, long, required = true)]
    pub source: String,

    #[arg(short, long, required = true)]
    pub destination: String,
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[command(subcommand)]
    pub command: Option<GenerateCommand>,
}

#[derive(Debug, Subcommand)]
pub enum GenerateCommand {
    Migrations {},
}
