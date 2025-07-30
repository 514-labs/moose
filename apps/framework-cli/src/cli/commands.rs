//! # CLI Commands
//! A module for all the commands that can be run from the CLI

use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    // Initializes the developer environment with all the necessary directories including temporary ones for data storage
    /// Initialize your data-intensive app or service
    Init {
        /// Name of your app or service
        name: String,

        /// Template to use for the project
        #[arg(
            conflicts_with = "from_remote",
            required_unless_present = "from_remote"
        )]
        template: Option<String>,

        /// Location of your app or service
        #[arg(short, long)]
        location: Option<String>,

        /// By default, the init command fails if the location directory exists, to prevent accidental reruns. This flag disables the check.
        #[arg(long)]
        no_fail_already_exists: bool,

        /// Initialize from a remote template repository
        #[arg(long, required_unless_present = "template")]
        from_remote: Option<String>,

        /// Programming language to use for the project
        #[arg(
            long,
            requires = "from_remote",
            required_unless_present = "template",
            conflicts_with = "template"
        )]
        language: Option<String>,
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

    /// View some data from a table or stream
    Peek {
        /// Name of the table or stream to peek
        name: String,
        /// Limit the number of rows to view
        #[arg(short, long, default_value = "5")]
        limit: u8,
        /// Output to a file
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// View data from a table
        #[arg(short = 't', long = "table", group = "resource_type")]
        table: bool,

        /// View data from a stream/topic
        #[arg(short = 's', long = "stream", group = "resource_type")]
        stream: bool,
    },
    /// Starts a local development environment to build your data-intensive app or service
    Dev {},
    /// Start a remote environment for use in cloud deployments
    Prod {
        /// Include and manage dependencies (ClickHouse, Redpanda, etc.) using Docker containers
        #[arg(long)]
        start_include_dependencies: bool,
    },
    /// Generates helpers for your data models (i.e. sdk, api tokens)
    Generate(GenerateArgs),
    /// Clears all temporary data and stops development infrastructure
    Clean {},
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
    /// Manage data processing workflows
    Workflow(WorkflowArgs),
    /// Manage templates
    Template(TemplateCommands),
    /// Integrate matching tables from a remote Moose instance into the local project
    Refresh {
        /// URL of the remote Moose instance (default: http://localhost:4000)
        #[arg(long)]
        url: Option<String>,

        /// API token for authentication with the remote Moose instance
        /// This token will be sent as a Bearer token in the Authorization header
        #[arg(long)]
        token: Option<String>,
        // #[arg(default_value = "true", short, long)]
        // interactive: bool,
    },
    /// Seed data into your project
    Seed(SeedCommands),
}

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[command(subcommand)]
    pub command: Option<GenerateCommand>,
}

#[derive(Debug, Subcommand)]
pub enum GenerateCommand {
    HashToken {},
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
    /// List registered workflows
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Show workflow history
    History {
        /// Filter workflows by status (running, completed, failed)
        #[arg(short, long)]
        status: Option<String>,

        /// Limit the number of workflows shown
        #[arg(short, long, default_value = "10")]
        limit: u32,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    /// Terminate a workflow
    #[command(hide = true)]
    Terminate {
        /// Name of the workflow to terminate
        name: String,
    },
    /// Cancel a workflow & allow tasks to execute cleanup
    Cancel {
        /// Name of the workflow to cancel
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

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct TemplateCommands {
    #[command(subcommand)]
    pub command: Option<TemplateSubCommands>,
}

#[derive(Debug, Subcommand)]
pub enum TemplateSubCommands {
    /// List available templates
    List {},
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
pub struct SeedCommands {
    #[command(subcommand)]
    pub command: Option<SeedSubcommands>,
}

#[derive(Debug, Subcommand)]
pub enum SeedSubcommands {
    /// Seed ClickHouse tables with data
    Clickhouse {
        /// ClickHouse connection string (e.g. clickhouse://user:pass@host:port/db)
        #[arg(long, value_name = "CONNECTION_STRING")]
        connection_string: String,
        /// Limit the number of rows to copy per table (default: 1000)
        #[arg(long, value_name = "LIMIT", default_value_t = 1000)]
        limit: usize,
        /// Only seed a specific table (optional)
        #[arg(long, value_name = "TABLE_NAME")]
        table: Option<String>,
    },
}
