//! # CLI Commands
//! A module for all the commands that can be run from the CLI

use std::path::PathBuf;

use crate::framework::languages::SupportedLanguages;
use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    // Initializes the developer environment with all the necessary directories including temporary ones for data storage
    /// Initialize your data-intensive app or service
    Init {
        /// Name of your app or service
        name: String,

        /// Template to use for the project
        template: String,

        /// Location of your app or service
        #[arg(short, long)]
        location: Option<String>,

        /// By default, the init command fails if the location directory exists, to prevent accidental reruns. This flag disables the check.
        #[arg(long)]
        no_fail_already_exists: bool,
    },
    /// Builds your moose project
    Build {
        /// Build for docker
        #[arg(short, long, default_value = "false")]
        docker: bool,
        /// Build for amd64 architecture
        #[arg(long)]
        amd64: bool,
        /// Build for arm64 architecture
        #[arg(long)]
        arm64: bool,
    },
    /// Checks the project for non-runtime errors
    Check {
        #[arg(long, default_value = "false")]
        write_infra_map: bool,
    },
    /// Displays the changes that will be applied to the infrastructure during the next deployment
    /// to production, considering the current state of the project
    Plan {
        /// URL of the remote Moose instance (default: http://localhost:4000)
        #[arg(long)]
        url: Option<String>,

        /// API token for authentication with the remote Moose instance
        /// This token will be sent as a Bearer token in the Authorization header
        #[arg(long)]
        token: Option<String>,
    },
    /// Starts a local development environment to build your data-intensive app or service
    Dev {},
    /// Start a remote environment for use in cloud deployments
    Prod {},
    /// Generates helpers for your data models (i.e. sdk, api tokens)
    Generate(GenerateArgs),
    /// Clears all temporary data and stops development infrastructure
    Clean {},
    /// Transforms upstream data into materialized datasets for analysis
    Function(FunctionArgs),
    /// Shapes & manipulates batches of data using SQL
    Block(BlockArgs),
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
        #[arg(short, long, default_value = "10", hide = true)]
        limit: u16,

        /// View a specific version of data models & database infrastructure (default: latest)
        #[arg(short, long, hide = true)]
        version: Option<String>,

        /// View streaming topics
        #[arg(short, long, default_value = "false", hide = true)]
        streaming: bool,

        /// Filter by infrastructure type (tables, streams, ingestion, sql_resource, consumption)
        #[arg(long)]
        _type: Option<String>,

        /// Filter by name (supports partial matching)
        #[arg(long)]
        name: Option<String>,

        /// Output results in JSON format
        #[arg(long, default_value = "false")]
        json: bool,
    },

    /// Opens metrics console for viewing live metrics from your moose app
    Metrics {},
    Import {
        data_model_name: String,
        #[arg(short, long)]
        file: PathBuf,
        #[arg(long)]
        format: Option<String>,
        #[arg(long, default_value = "http://localhost:4000/ingest")]
        destination: String,
        #[arg(long)]
        version: Option<String>,
    },
    DataModel(DataModelArgs),
    /// View some data from a data model
    Peek {
        data_model_name: String,
        /// Limit the number of rows to view
        #[arg(short, long, default_value = "5")]
        limit: u8,
        /// Output to a file
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// peek from the topic instead of the table
        #[arg(long)]
        topic: bool,
    },
    /// Manage data processing workflows
    Workflow(WorkflowArgs),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[command(subcommand)]
    pub command: Option<GenerateCommand>,
}

#[derive(Debug, Subcommand)]
pub enum GenerateCommand {
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
        /// Force overwrite of files and folders in the directory
        #[arg(default_value = "false", short = 'y', long)]
        overwrite: bool,
    },
    HashToken {},
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
pub struct BlockArgs {
    #[command(subcommand)]
    pub command: Option<BlockCommands>,
}

#[derive(Debug, Subcommand)]
pub enum BlockCommands {
    /// Creates a starter block
    #[command(arg_required_else_help = true)]
    Init {
        /// Name of your block
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

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct DataModelArgs {
    #[command(subcommand)]
    pub command: Option<DataModelCommands>,
}

#[derive(Debug, Subcommand)]
pub enum DataModelCommands {
    #[command(arg_required_else_help = true)]
    Init(DataModelInitArgs),
}

#[derive(Debug, Args)]
pub struct DataModelInitArgs {
    pub name: String,

    /// Name of your sample file
    #[arg(short, long)]
    pub sample: String,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    pub command: Option<WorkflowCommands>,
}

#[derive(Debug, Subcommand)]
pub enum WorkflowCommands {
    /// Initialize a new workflow
    Init {
        /// Name of your workflow
        name: String,

        /// Comma-separated list of task names
        #[arg(long)]
        tasks: Option<String>,

        /// Individual task names (can be specified multiple times)
        #[arg(long)]
        task: Option<Vec<String>>,
    },
    /// Run a workflow
    Run {
        /// Name of the workflow to run
        name: String,

        /// JSON input parameters for the workflow
        #[arg(short, long)]
        input: Option<String>,
    },
    /// Resume a workflow from a specific task
    Resume {
        /// Name of the workflow to resume
        name: String,

        /// Task to resume from
        #[arg(long)]
        from: String,
    },
    /// List running workflows
    List {
        /// Filter workflows by status (running, completed, failed)
        #[arg(short, long)]
        status: Option<String>,

        /// Limit the number of workflows shown
        #[arg(short, long, default_value = "10")]
        limit: u32,
    },
    /// Terminate a workflow
    Terminate {
        /// Name of the workflow to terminate
        name: String,
    },
    /// Pause a workflow
    Pause {
        /// Name of the workflow to pause
        name: String,
    },
    /// Unpause a workflow
    Unpause {
        /// Name of the workflow to unpause
        name: String,
    },
    /// Get the status of a workflow
    Status {
        /// Name of the workflow
        name: String,

        /// Optional run ID (defaults to most recent)
        #[arg(long)]
        id: Option<String>,

        /// Verbose output
        #[arg(long)]
        verbose: bool,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
}
