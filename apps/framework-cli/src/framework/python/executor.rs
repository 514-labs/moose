//! # Executes Python code in a subprocess.
//! This module provides a Python executor that can run Python code in a subprocess

use std::{
    path::Path,
    process::{Command, Stdio},
};

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

/// Executes a serializtion process to turn a Python file's contents into framework objects
pub fn serialize_contents(serializer: PythonSerializers, python_file: &Path) {
    let prgm = Command::new("python3")
        .arg(serializer.get_path())
        .arg(python_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute Python3");

    let output = prgm.wait_with_output().unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    println!("{}", output);
}

// TESTs
// #[cfg(test)]
// mod tests {
//     use super::*;
// }
