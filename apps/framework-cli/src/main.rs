#[macro_use]
mod cli;
pub mod framework;
pub mod infrastructure;
pub mod metrics;
pub mod metrics_inserter;
pub mod project;
pub mod utilities;

pub mod proto;

use std::io::Write;
use std::process::ExitCode;

use clap::Parser;
use cli::display::{Message, MessageType};

/// Ensures terminal is properly reset on exit  
fn ensure_terminal_cleanup() {
    // The nuclear option: use the system's reset command
    // This reads the terminal database and sends ALL necessary sequences
    let reset_result = std::process::Command::new("reset")
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    // If reset command succeeded, we're done
    if reset_result.is_ok() && reset_result.unwrap().success() {
        log::info!("Terminal cleanup complete via reset command");
        return;
    }

    // Fallback: try stty sane
    let _ = std::process::Command::new("stty")
        .arg("sane")
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    log::info!("Terminal cleanup complete via stty");
}

// Entry point for the CLI application
fn main() -> ExitCode {
    // Handle all CLI setup that doesn't require async functionality
    let user_directory = cli::settings::setup_user_directory();
    if let Err(e) = user_directory {
        show_message!(
            MessageType::Error,
            Message {
                action: "Init".to_string(),
                details: format!(
                    "Failed to initialize ~/.moose, please check your permissions: {e:?}"
                ),
            }
        );
        std::process::exit(1);
    }

    cli::settings::init_config_file().expect("Failed to init config file");
    let config = cli::settings::read_settings().expect("Failed to read settings");
    let machine_id = utilities::machine_id::get_or_create_machine_id();

    // Setup logging
    cli::logger::setup_logging(&config.logger, &machine_id).expect("Failed to setup logging");

    // Parse CLI arguments
    let cli_result = match cli::Cli::try_parse() {
        Ok(cli_result) => cli_result,
        Err(e) => {
            // For missing template argument, provide a helpful message
            if e.kind() == clap::error::ErrorKind::MissingRequiredArgument
                && e.to_string().contains("<TEMPLATE>")
            {
                eprintln!("{e}");
                eprintln!("To view available templates, run:");
                eprintln!("\n  moose template list");
                std::process::exit(1)
            } else {
                // For other errors, use Clap's default error format
                // this includes the --version and --help string
                e.exit()
            }
        }
    };

    if cli_result.backtrace {
        // Safe: no other threads have started and no errors have been created yet.
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    // Create a runtime with a single thread to avoid issues with dropping runtimes
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    // Run the async function to handle the command
    let result = runtime.block_on(cli::top_command_handler(
        config,
        &cli_result.command,
        machine_id,
    ));

    // Process the result using the original display formatting
    match result {
        Ok(s) => {
            show_message!(s.message_type, s.message);
            ensure_terminal_cleanup();
            return ExitCode::from(0);
        }
        Err(e) => {
            show_message!(e.message_type, e.message);
            if let Some(err) = e.error {
                eprintln!("{err:?}");
            }
            ensure_terminal_cleanup();
            return ExitCode::from(1);
        }
    }
}
