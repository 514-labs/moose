//! # Executes Python code in a subprocess.
//! This module provides a Python executor that can run Python code in a subprocess

use std::process::Stdio;

use tokio::process::{Child, Command};

/// Checks if the Python interpreter is available

/// Checks the version of the Python interpreter

/// Ensures that Python3.7 is available on the system

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

#[derive(Debug, Clone)]
pub enum PythonProgram {
    FlowRunner { args: Vec<String> },
}

impl PythonProgram {
    pub fn get_path(&self) -> &str {
        "src/framework/python/scripts/flow_runner.py"
    }
}

/// Executes a Python program in a subprocess
pub fn run_python_program(program: PythonProgram) -> Result<Child, std::io::Error> {
    let get_args = match program.clone() {
        PythonProgram::FlowRunner { args } => args,
    };

    Command::new("python3")
        .arg(program.get_path())
        .args(get_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

// TESTs
// #[cfg(test)]
// mod tests {
//     use super::*;
// }
