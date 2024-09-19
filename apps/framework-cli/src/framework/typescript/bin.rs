use crate::{cli::display::MessageType, utilities::constants::TSCONFIG_JSON};
use serde::Deserialize;
use std::{path::Path, process::Stdio};

use tokio::process::{Child, Command};

#[derive(Deserialize)]
pub struct CliMessage {
    pub message_type: MessageType,
    pub action: String,
    pub message: String,
}

const RUNNER_COMMAND: &str = "moose-runner";

pub fn run(
    binary_command: &str,
    project_path: &Path,
    args: &[&str],
) -> Result<Child, std::io::Error> {
    let mut command = Command::new(RUNNER_COMMAND);

    command.arg(binary_command);

    command.env("TS_NODE_PROJECT", project_path.join(TSCONFIG_JSON));

    for arg in args {
        command.arg(arg);
    }

    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
