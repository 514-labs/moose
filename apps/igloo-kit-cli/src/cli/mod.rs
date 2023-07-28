mod commands;
mod routines;

use commands::Commands;
use std::path::PathBuf;

use clap::Parser;

use crate::framework::AddableObjects;

use self::commands::AddArgs;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn add_handler(add_arg: &AddArgs) {
    match &add_arg.command {
        Some(AddableObjects::IngestPoint) => {
            println!("Adding ingestion point...");
        }
        Some(AddableObjects::Flow) => {
            println!("Adding flow...");
        }
        Some(AddableObjects::Dataframe) => {
            println!("Adding dataframe...");
        }
        Some(AddableObjects::Metric) => {
            println!("Adding metric...");
        }
        Some(AddableObjects::Dashboard) => {
            println!("Adding dashboard...");
        }
        Some(AddableObjects::Model) => {
            println!("Adding model...");
        }
        None => {}
    }
}

fn top_command_handler(commands: &Option<Commands>) {
    match commands {
        Some(Commands::Init {}) => {
            routines::initialize_project();
        }
        Some(Commands::Dev{}) => {
            println!("Starting development environment...");
        }
        Some(Commands::Update{}) => {
            println!("Updating...");
        }
        Some(Commands::Stop{}) => {
            println!("Stopping...");
        }
        Some(Commands::Clean{}) => {
            println!("Cleaning...");
        }
        Some(Commands::Add(add_arg)) => {
            add_handler(&add_arg)
        }
        None => {}
    }
}

pub fn cli_run() {
    // Validate the CLI is running in a project directory or that a project must be created

    let cli = Cli::parse();

    if let Some(config_path) = cli.config.as_deref() {
        println!("Value for config: {}", config_path.display());
    }

    // You can see how many times a particular flag or argument occurred
    // Note, only flags can have multiple occurrences
    match cli.debug {
        0 => println!("Debug mode is off"),
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    top_command_handler(&cli.command)
}
