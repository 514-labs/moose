mod commands;
mod routines;
pub mod user_messages;

use commands::Commands;
use std::path::PathBuf;
use clap::Parser;
use crate::{framework::{AddableObjects, directories::get_igloo_directory, self}, infrastructure};
use self::{commands::AddArgs, user_messages::{MessageType, Message}};

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
            todo!("add ingestion point object");
        }
        Some(AddableObjects::Flow) => {
            todo!("add flow object");
        }
        Some(AddableObjects::Dataframe) => {
            todo!("add dataframe object");
        }
        Some(AddableObjects::Metric) => {
            todo!("add metric object");
        }
        Some(AddableObjects::Dashboard) => {
            todo!("add dashboard object")
        }
        Some(AddableObjects::Model) => {
            todo!("add model object")
        }
        None => {
            todo!("add a great, piffy, and helpful message here")
        }
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

fn top_command_handler(commands: &Option<Commands>, debug: bool, igloo_dir: PathBuf) {
    let mut term: CommandTerminal = CommandTerminal::new();

    match commands {
        Some(Commands::Init {}) => {
            routines::initialize_project(&mut term);
        }
        Some(Commands::Dev{}) => {
            routines::start_containers(&mut term);
            infrastructure::setup::validate::validate_red_panda_cluster(&mut term, debug);
        }
        Some(Commands::Update{}) => {
            todo!("Will update the project's underlying infrascructure based on any added objects")
        }
        Some(Commands::Stop{}) => {
            routines::stop_containers(&mut term);
        }
        Some(Commands::Clean{}) => {
            routines::clean_project(&mut term, &igloo_dir);

        }
        Some(Commands::Add(add_args)) => {
            
            add_handler(add_args);   
        }
        None => {}
    }
    
}

pub fn cli_run() {
    // Validate the CLI is running in a project directory or that a project must be created

    let cli = Cli::parse();

    let igloo_dir = cli.config.unwrap_or(get_igloo_directory().unwrap());

    top_command_handler(&cli.command, cli.debug, igloo_dir)
}
