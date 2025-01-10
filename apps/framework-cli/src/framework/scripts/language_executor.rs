use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::framework::languages::SupportedLanguages;

#[derive(Debug, serde::Deserialize)]
struct PythonError {
    error: String,
    details: String,
    file: Option<String>,
}

async fn execute_python_script(
    script_path: &PathBuf,
    input_data: Option<serde_json::Value>,
) -> Result<Option<serde_json::Value>> {
    println!("Executing script: {:?}", script_path);

    let mut command = Command::new("python");

    // Create temporary wrapper file
    let temp_dir = tempfile::tempdir()?;
    let wrapper_path = temp_dir.path().join("wrapper.py");
    fs::write(&wrapper_path, include_str!("./wrappers/python_wrapper.py"))?;

    command.arg(&wrapper_path).arg(script_path);

    if let Some(input) = input_data {
        command.env("MOOSE_INPUT", serde_json::to_string(&input)?);
    }

    let output = command
        .output()
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

/// Execute a specific script
async fn execute_script(
    language: SupportedLanguages,
    script_path: &PathBuf,
    input_data: Option<serde_json::Value>,
) -> Result<Option<serde_json::Value>> {
    match language {
        SupportedLanguages::Python => execute_python_script(script_path, input_data).await,
        _ => Err(anyhow::anyhow!("Unsupported language: {:?}", language)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_sequential_workflow() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create scripts directory structure
        fs::create_dir(temp_dir.path().join("scripts"))?;

        fs::write(
            temp_dir.path().join("scripts").join("1.extract.py"),
            r#"
from moose_lib import task

@task
def extract():
    return {"step": "extract", "data": [1, 2, 3]}
            "#,
        )?;

        fs::write(
            temp_dir.path().join("scripts").join("2.transform.py"),
            r#"
from moose_lib import task

@task
def transform():
    return {"step": "transform", "data": [2, 4, 6]}
            "#,
        )?;

        let result = execute_script(
            SupportedLanguages::Python,
            &temp_dir.path().join("scripts"),
            None,
        )
        .await?;

        assert!(result.is_some());
        let results = result.unwrap();
        assert_eq!(results.as_array().unwrap().len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_parallel_workflow() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create scripts directory structure with parallel steps
        fs::create_dir_all(temp_dir.path().join("scripts/2.parallel"))?;

        fs::write(
            temp_dir.path().join("scripts").join("1.extract.py"),
            r#"
from moose_lib import task

@task
def extract():
    return {"step": "extract", "data": [1, 2, 3]}
            "#,
        )?;

        fs::write(
            temp_dir
                .path()
                .join("scripts/2.parallel")
                .join("process_a.py"),
            r#"
from moose_lib import task

@task
def process_a():
    return {"step": "process_a", "data": [2, 4, 6]}
            "#,
        )?;

        fs::write(
            temp_dir
                .path()
                .join("scripts/2.parallel")
                .join("process_b.py"),
            r#"
from moose_lib import task

@task
def process_b():
    return {"step": "process_b", "data": [3, 6, 9]}
            "#,
        )?;

        let result = execute_script(
            SupportedLanguages::Python,
            &temp_dir.path().join("scripts"),
            None,
        )
        .await?;

        assert!(result.is_some());
        let results = result.unwrap();
        assert_eq!(results.as_array().unwrap().len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_file_type() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create a TypeScript file in Python scripts
        fs::create_dir(temp_dir.path().join("scripts"))?;
        fs::write(
            temp_dir.path().join("scripts").join("1.process.ts"),
            "console.log('This should fail');",
        )?;

        let result = execute_script(
            SupportedLanguages::Python,
            &temp_dir.path().join("scripts"),
            None,
        )
        .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid file type"));

        Ok(())
    }
}
