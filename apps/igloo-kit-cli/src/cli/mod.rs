mod commands;
mod routines;
mod user_messages;

use commands::Commands;
use std::path::PathBuf;
use clap::Parser;
use crate::framework::AddableObjects;
use self::{commands::AddArgs, user_messages::{MessageType, Message, show_message}, routines::initialize_project};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn add_handler(add_arg: &AddArgs) {
    match &add_arg.command {
        Some(AddableObjects::IngestPoint) => {
            println!("Adding ingest point...")
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

pub struct CommandTerminal {
    term: console::Term,
    counter: usize,
}

impl CommandTerminal {
    pub fn new() -> CommandTerminal {
        CommandTerminal {
            term: console::Term::stdout(),
            counter: 0,
        }
    }

    pub fn clear(&mut self) {
        self.term.clear_last_lines(self.counter).expect("failed to clear the terminal");
        self.counter = 0;
    }

    pub fn clear_with_delay(&mut self, delay: u64) {
        std::thread::sleep(std::time::Duration::from_millis(delay));
        self.clear();
    }
}

fn top_command_handler(commands: &Option<Commands>, debug: bool) {
    match commands {
        Some(Commands::Init {}) => {
            let mut term: CommandTerminal = CommandTerminal::new();
            routines::initialize_project(&mut term);
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

    top_command_handler(&cli.command, cli.debug)
}
