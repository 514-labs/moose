//! # Executes Python code in a subprocess.
//! This module provides a Python executor that can run Python code in a subprocess

use std::path::Path;
use std::process::Stdio;

use crate::utilities::constants::{CLI_INTERNAL_VERSIONS_DIR, CLI_PROJECT_INTERNAL_DIR};

use tokio::process::{Child, Command};

pub enum PythonSerializers {
    FrameworkObjectSerializer,
    ProjectObjectSerializer,
}

impl PythonSerializers {
    pub fn get_path(&self) -> &str {
        match self {
            PythonSerializers::FrameworkObjectSerializer => {
                "src/framework/python/scripts/framework_object_serializer.py"
            }
            PythonSerializers::ProjectObjectSerializer => {
                "src/framework/python/scripts/project_object_serializer.py"
            }
        }
    }
}

// TODO: move to PythonCommands
#[derive(Debug, Clone)]
pub enum PythonProgram {
    StreamingFunctionRunner { args: Vec<String> },
    BlocksRunner { args: Vec<String> },
    ConsumptionRunner { args: Vec<String> },
    LoadApiParam { args: Vec<String> },
    OrchestrationWorker { args: Vec<String> },
}

/// executable files in the python moose-lib
pub enum PythonCommand {
    DmV2Serializer,
}

pub static STREAMING_FUNCTION_RUNNER: &str = include_str!("wrappers/streaming_function_runner.py");
pub static BLOCKS_RUNNER: &str = include_str!("wrappers/blocks_runner.py");
pub static CONSUMPTION_RUNNER: &str = include_str!("wrappers/consumption_runner.py");
pub static LOAD_API_PARAMS: &str = include_str!("wrappers/load_api_params.py");
pub static ORCHESTRATION_WORKER: &str = include_str!("wrappers/scripts/worker-main.py");

const PYTHON_PATH: &str = "PYTHONPATH";
fn python_path_with_version() -> String {
    let mut paths = std::env::var(PYTHON_PATH).unwrap_or_else(|_| String::from(""));
    if !paths.is_empty() {
        paths.push(':');
    }
    paths.push_str(CLI_PROJECT_INTERNAL_DIR);
    paths.push('/');
    paths.push_str(CLI_INTERNAL_VERSIONS_DIR);

    paths.push(':');
    paths.push_str(CLI_PROJECT_INTERNAL_DIR);

    paths
}

/// Executes a Python program in a subprocess
pub fn run_python_command(command: PythonCommand) -> Result<Child, std::io::Error> {
    let (get_args, library_module) = match command {
        PythonCommand::DmV2Serializer => (Vec::<String>::new(), "moose_lib.dmv2-serializer"),
    };

    Command::new("python3")
        .env(PYTHON_PATH, python_path_with_version())
        .arg("-m")
        .arg(library_module)
        .args(get_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

/// Executes a Python program in a subprocess
pub fn run_python_program(program: PythonProgram) -> Result<Child, std::io::Error> {
    let (get_args, program_string) = match program {
        PythonProgram::StreamingFunctionRunner { args } => (args, STREAMING_FUNCTION_RUNNER),
        PythonProgram::BlocksRunner { args } => (args, BLOCKS_RUNNER),
        PythonProgram::ConsumptionRunner { args } => (args, CONSUMPTION_RUNNER),
        PythonProgram::LoadApiParam { args } => (args, LOAD_API_PARAMS),
        PythonProgram::OrchestrationWorker { args } => (args, ORCHESTRATION_WORKER),
    };

    Command::new("python3")
        .env(PYTHON_PATH, python_path_with_version())
        .arg("-u")
        .arg("-c")
        .arg(program_string)
        .args(get_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub async fn run_python_file(path: &Path, env: &[(&str, &str)]) -> Result<Child, std::io::Error> {
    let mut command = Command::new("python3");

    command.env(PYTHON_PATH, python_path_with_version());
    for (key, val) in env {
        command.env(key, val);
    }

    command
        .arg("-u")
        .arg(path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
}

pub fn add_optional_arg(args: &mut Vec<String>, flag: &str, value: &Option<String>) {
    if let Some(val) = value {
        args.push(flag.to_string());
        args.push(val.to_string());
    }
}

// TESTs
#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::infrastructure::stream::redpanda::RedpandaConfig;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_run_python_flow_runner_program() {
        let redpanda_config = RedpandaConfig::default();
        let source_topic = "UserActivity_0_0";
        let target_topic = "ParsedActivity_0_0";
        let flow_path = Path::new(
            "/Users/timdelisle/Dev/igloo-stack/apps/framework-cli/tests/python/flows/valid",
        );

        let program = PythonProgram::StreamingFunctionRunner {
            args: vec![
                source_topic.to_string(),
                target_topic.to_string(),
                flow_path.to_str().unwrap().to_string(),
                redpanda_config.broker,
            ],
        };

        let child = run_python_program(program).unwrap();
        let output = child.wait_with_output().await.unwrap();

        //print output stdout and stderr
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}
