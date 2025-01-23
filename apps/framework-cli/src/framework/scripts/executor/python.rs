use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, serde::Deserialize)]
struct PythonError {
    error: String,
    details: String,
    // file: Option<String>,
}

pub async fn execute_workflow(
    script_path: &PathBuf,
    input_data: Option<serde_json::Value>,
) -> Result<Option<serde_json::Value>> {
    println!("Executing script: {:?}", script_path);

    let mut command = Command::new("python");

    // Create temporary directory
    let temp_dir = tempfile::tempdir()?;

    // TODO: Move all of this to the python lib?
    // Copy the entire python_wrapper package
    let wrapper_dir = temp_dir.path().join("python_wrapper");
    fs::create_dir(&wrapper_dir)?;

    // Copy all wrapper files
    fs::write(
        wrapper_dir.join("__init__.py"),
        include_str!("../wrappers/python_wrapper/__init__.py"),
    )?;
    fs::write(
        wrapper_dir.join("activity.py"),
        include_str!("../wrappers/python_wrapper/activity.py"),
    )?;
    fs::write(
        wrapper_dir.join("logger.py"),
        include_str!("../wrappers/python_wrapper/logger.py"),
    )?;
    fs::write(
        wrapper_dir.join("worker.py"),
        include_str!("../wrappers/python_wrapper/worker.py"),
    )?;
    fs::write(
        wrapper_dir.join("workflow.py"),
        include_str!("../wrappers/python_wrapper/workflow.py"),
    )?;

    // Create and write the wrapper script
    let wrapper_path = temp_dir.path().join("wrapper.py");
    fs::write(&wrapper_path, include_str!("../wrappers/python_wrapper.py"))?;

    command.arg(&wrapper_path).arg(script_path);

    if let Some(input) = input_data {
        command.env("MOOSE_INPUT", serde_json::to_string(&input)?);
    }

    let output = command
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute script: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        // Try to parse as structured error
        if let Ok(python_error) = serde_json::from_str::<PythonError>(&error) {
            match python_error.error.as_str() {
                "InvalidFileType" => {
                    return Err(anyhow::anyhow!(
                        "Invalid file type: {}",
                        python_error.details
                    ))
                }
                _ => return Err(anyhow::anyhow!("Script failed: {}", python_error.details)),
            }
        }

        return Err(anyhow::anyhow!("Script failed: {}", error));
    }

    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(serde_json::from_str(&stdout)?))
    }
}
