use crate::{cli::display::MessageType, utilities::constants::TSCONFIG_JSON};
use serde::Deserialize;
use std::{env, path::Path, process::Stdio};

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

    // This adds the node_modules/.bin to the PATH so that we can run moose-tspc
    let path = env::var("PATH").unwrap_or_else(|_| "/usr/local/bin".to_string());
    let bin_path = format!(
        "{}:{}/node_modules/.bin",
        path,
        project_path.to_str().unwrap()
    );

    command
        .env("TS_NODE_PROJECT", project_path.join(TSCONFIG_JSON))
        .env("PATH", bin_path)
        .env("TS_NODE_COMPILER_HOST", "true")
        .env("TS_NODE_EMIT", "true");
    if binary_command == "consumption-apis" {
        command.env("TS_NODE_COMPILER", "ts-patch/compiler");
    }

    for arg in args {
        command.arg(arg);
    }

    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}
