//! # CLI Commands
//! A module for all the commands that can be run from the CLI

use std::path::PathBuf;

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
        #[arg(short, long)]
        location: Option<String>,

        /// Template to use for the project
        #[arg(short, long)]
        template: Option<String>,

        /// By default, the init command fails if the location directory exists, to prevent accidental reruns. This flag disables the check.
        #[arg(long)]
        no_fail_already_exists: bool,

        /// Create basic Moose project without examples
        #[arg(short, long, default_value = "false")]
        empty: bool,
    },
    /// Builds your moose project
    Build {
        #[arg(short, long)]
        docker: bool,
    },
    /// [Not Ready] Displays the changes that will be applied to the infrastructure during the next deployment
    /// to production, consdering the current state of the project
    Plan {},
    /// Starts a local development environment to build your data-intensive app or service
    Dev {},
    /// Start a remote environment for use in cloud deployments
    Prod {},
    /// Generates missing migration files
    Generate(GenerateArgs),
    /// Bumps the version of the project
    BumpVersion { new_version: Option<String> },
    /// Clears all temporary data and stops development infrastructure
    Clean {},
    /// Transforms upstream data into materialized datasets for analysis
    Function(FunctionArgs),
    /// Defines aggregate table views of upstream data models
    Aggregation(AggregationArgs),
    /// Defines consumption APIs
    Consumption(ConsumptionArgs),
    /// View Moose logs
    Logs {
        /// Follow the logs in real-time
        #[arg(short, long)]
        tail: bool,

        /// Filter logs by a specific string
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// View Moose processes
    Ps {},
    /// View Moose primitives & infrastructure
    Ls {
        /// Limit output to a specific number of data models
        #[arg(short, long, default_value = "10")]
        limit: u16,

        /// View a specific version of data models & database infrastructure (default: latest)
        #[arg(short, long)]
        version: Option<String>,

        /// View streaming topics
        #[arg(short, long, default_value = "false")]
        streaming: bool,
    },
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[command(subcommand)]
    pub command: Option<GenerateCommand>,
}

#[derive(Debug, Subcommand)]
pub enum GenerateCommand {
    Migrations {},
    Sdk {
        /// Language of the SDK to be generated
        #[arg(default_value_t = SupportedLanguages::Typescript, value_enum, short, long)]
        language: SupportedLanguages,
        /// Where the SDK files should be written to
        #[arg(default_value = "./sdk", short, long)]
        destination: PathBuf,
        /// The location of the Moose project
        #[arg(default_value = ".", short, long)]
        project_location: PathBuf,
        /// Whether or not to generate a full fledged package or just the source files in the language of choice
        #[arg(default_value = "false", short = 'f', long)]
        full_package: bool,
    },
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct FunctionArgs {
    #[command(subcommand)]
    pub command: Option<FunctionCommands>,
}

#[derive(Debug, Subcommand)]
pub enum FunctionCommands {
    /// Structures the project's directory & files for a new streaming function
    #[command(arg_required_else_help = true)]
    Init(FuncInitArgs),
}

#[derive(Debug, Args)]
pub struct FuncInitArgs {
    /// Name of your source data model
    #[arg(short, long, required = true)]
    pub source: String,

    /// Name of your destination data model
    #[arg(short, long, required = true)]
    pub destination: String,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct AggregationArgs {
    #[command(subcommand)]
    pub command: Option<AggregationCommands>,
}

#[derive(Debug, Subcommand)]
pub enum AggregationCommands {
    /// Creates a starter aggregation
    #[command(arg_required_else_help = true)]
    Init {
        /// Name of your aggregation
        name: String,
    },
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct ConsumptionArgs {
    #[command(subcommand)]
    pub command: Option<ConsumptionCommands>,
}

#[derive(Debug, Subcommand)]
pub enum ConsumptionCommands {
    /// Creates a starter api
    #[command(arg_required_else_help = true)]
    Init {
        /// Name of your api
        name: String,
    },
}
