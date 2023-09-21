mod commands;
mod routines;
mod config;
mod watcher;
mod webserver;
pub mod user_messages;

use commands::Commands;
use config::{read_config, Config};
use clap::Parser;
use crate::{framework::{AddableObjects, directories::get_igloo_directory, schema::parse_schema_file}, infrastructure::{self, db::{clickhouse::ClickhouseConfig, self}, PANDA_NETWORK}};
use self::{commands::AddArgs, user_messages::{MessageType, Message, show_message}};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    // TODD: Add a config file option
    /// Sets a custom config file
    // #[arg(short, long, value_name = "FILE")]
    // config: Option<PathBuf>,

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

async fn top_command_handler(term: &mut CommandTerminal, config: Config, commands: &Option<Commands>, debug: bool) {
    let clickhouse_config = ClickhouseConfig {
        db_name: "local".to_string(),
        user: "panda".to_string(),
        password: "pandapass".to_string(),
        host: "localhost".to_string(),
        host_port: 18123,
        postgres_port: 9005,
        kafka_port: 9092,
        cluster_network: PANDA_NETWORK.to_owned(),
    };

    if !config.features.coming_soon_wall {
        match commands {
            Some(Commands::Init {}) => {
                routines::initialize_project(term);
            }
            Some(Commands::Dev{}) => {
                routines::start_containers(term, clickhouse_config.clone());
                infrastructure::setup::validate::validate_red_panda_cluster(term, debug);
                routines::start_development_mode(term, clickhouse_config.clone()).await;      
            }
            Some(Commands::Update{}) => {
                // This command may not be needed if we have incredible automation
                todo!("Will update the project's underlying infrascructure based on any added objects")
            }
            Some(Commands::Stop{}) => {
                routines::stop_containers(term);
            }
            Some(Commands::Clean{}) => {
                let igloo_dir = get_igloo_directory().expect("Nothing to clean, no .igloo directory found");
                routines::clean_project(term, &igloo_dir);

                }
                Some(Commands::Add(add_args)) => {
                    add_handler(add_args);   
                }
                None => {}
        }
    } else {
        show_message(term, MessageType::Banner, Message {
            action: "Coming Soon",
            details: "Join the IglooKit community to stay up to date on the latest features: https://discord.gg/WX3V3K4QCc",
        });
    }
    
}

pub async fn cli_run() {
    let mut term: CommandTerminal = CommandTerminal::new();
    let config = read_config(&mut term);
    let cli = Cli::parse();

    // let igloo_dir = cli.config;

    top_command_handler(&mut term, config, &cli.command, cli.debug).await
}
