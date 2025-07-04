//! # Executes Python code in a subprocess.
//! This module provides a Python executor that can run Python code in a subprocess

use std::path::Path;
use std::process::Stdio;

use crate::utilities::constants::{CLI_PROJECT_INTERNAL_DIR, LIB_DIR};

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
    ConsumptionRunner { args: Vec<String> },
    LoadApiParam { args: Vec<String> },
    OrchestrationWorker { args: Vec<String> },
}

/// executable files in the python moose-lib
pub enum PythonCommand {
    DmV2Serializer,
    StreamingFunctionRunner { args: Vec<String> },
}

pub static CONSUMPTION_RUNNER: &str = include_str!("wrappers/consumption_runner.py");
pub static LOAD_API_PARAMS: &str = include_str!("wrappers/load_api_params.py");
pub static ORCHESTRATION_WORKER: &str = include_str!("wrappers/scripts/worker-main.py");

const PYTHON_PATH: &str = "PYTHONPATH";

/// Gets the Python path including the lib directory if it exists in the current project.
///
/// This function builds the PYTHONPATH environment variable by:
/// 1. Starting with the existing PYTHONPATH if any
/// 2. Adding the CLI project internal directory
/// 3. Adding the lib directory if it exists in the current working directory
///
/// # Returns
///
/// A String containing the complete PYTHONPATH with all necessary directories
fn python_path_with_lib(project_location: &Path) -> String {
    // Start with existing PYTHONPATH if any
    let mut paths = std::env::var(PYTHON_PATH).unwrap_or_else(|_| String::from(""));

    // Check if lib directory exists in current directory
    let lib_path = project_location.join(LIB_DIR);
    if lib_path.exists() && lib_path.is_dir() {
        paths.push(':');
        if let Some(lib_path_str) = lib_path.to_str() {
            paths.push_str(lib_path_str);
        }
    }

    // There's existing logic that write shared python modules into the project's
    // internal directory (e.g. temporal utils).
    // TODO: Move all python scripts to py-moose-lib
    paths.push(':');
    paths.push_str(CLI_PROJECT_INTERNAL_DIR);

    paths
}

/// Executes a Python program in a subprocess
pub fn run_python_command(
    project_location: &Path,
    command: PythonCommand,
) -> Result<Child, std::io::Error> {
    let (get_args, library_module) = match command {
        PythonCommand::DmV2Serializer => (Vec::<String>::new(), "moose_lib.dmv2-serializer"),
        PythonCommand::StreamingFunctionRunner { args } => {
            (args, "moose_lib.streaming.streaming_function_runner")
        }
    };

    Command::new("python3")
        .env(PYTHON_PATH, python_path_with_lib(project_location))
        .arg("-m")
        .arg(library_module)
        .args(get_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

/// Executes a Python program in a subprocess
pub fn run_python_program(
    project_location: &Path,
    program: PythonProgram,
) -> Result<Child, std::io::Error> {
    let (get_args, program_string) = match program {
        PythonProgram::ConsumptionRunner { args } => (args, CONSUMPTION_RUNNER),
        PythonProgram::LoadApiParam { args } => (args, LOAD_API_PARAMS),
        PythonProgram::OrchestrationWorker { args } => (args, ORCHESTRATION_WORKER),
    };

    Command::new("python3")
        .env(PYTHON_PATH, python_path_with_lib(project_location))
        .arg("-u")
        .arg("-c")
        .arg(program_string)
        .args(get_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

pub async fn run_python_file(
    project_location: &Path,
    path: &Path,
    env: &[(&str, &str)],
) -> Result<Child, std::io::Error> {
    let mut command = Command::new("python3");

    command.env(PYTHON_PATH, python_path_with_lib(project_location));
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
