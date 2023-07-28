mod infrastructure;
use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

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

#[derive(Subcommand)]
enum Commands {
    // Initializes the developer environment with all the necessary directories including temporary ones for data storage
    Init {
        #[arg(short, long)]
        list: bool,
    },
    // Adds a new templated object to the project
    Add(AddArgs),
    // Spins up development infrastructure including a redpanda cluster and clickhouse database
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
struct AddArgs {
    #[command(subcommand)]
    command: Option<AddableObjects>,
}

#[derive(Debug, Subcommand)]
enum AddableObjects {
    IngestPoint,
    Dataframe,
    Metric,
    Dashboard,
    Model,

}

fn cli_run() {
    let cli = Cli::parse();

    // You can check the value provided by positional arguments, or option arguments
    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

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

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Some(Commands::Init { list }) => {
            if *list {
                println!("Printing testing lists...");
            } else {
                println!("Not printing testing lists...");
            }
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
            match &add_arg.command {
                Some(AddableObjects::IngestPoint) => {
                    println!("Adding ingestion point...");
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
        None => {}
    }
}

fn main() {
    infrastructure::init();
}

