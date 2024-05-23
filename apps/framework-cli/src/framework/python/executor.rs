//! # Executes Python code in a subprocess.
//! This module provides a Python executor that can run Python code in a subprocess

use std::process::{Command, Stdio};

/// Checks if the Python interpreter is available

/// Checks the version of the Python interpreter

/// Ensures that Python3.7 is available on the system

/// Executes a serializtion process to turn a Python file's contents into framework objects
fn serialize_contents() {
    let prgm = Command::new("python3")
        .arg("src/framework/python/scripts/framework_object_serializer.py")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute Python3");

    let output = prgm.wait_with_output().unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    println!("{}", output);
}

// TESTs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_contents() {
        serialize_contents();
    }
}
