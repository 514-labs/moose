use crate::cli::display::MessageType;
use serde::Deserialize;
use std::process::Stdio;

use tokio::process::{Child, Command};

#[derive(Deserialize)]
pub struct CliMessage {
    pub message_type: MessageType,
    pub action: String,
    pub message: String,
}

pub fn run(script: &str, args: &[&str]) -> Result<Child, std::io::Error> {
    let mut command = Command::new("npx");

    command
        .arg("--yes")
        .arg("ts-node")
        .arg("--skipProject")
        .arg("-e")
        .arg(script)
        .arg("--");

    for arg in args {
        command.arg(arg);
    }

    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
