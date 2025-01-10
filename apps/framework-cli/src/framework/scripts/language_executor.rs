use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::framework::languages::SupportedLanguages;

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

    // Add input data as environment variable
    if let Some(input) = input_data {
        command.env("MOOSE_INPUT", serde_json::to_string(&input)?);
    }

    // Execute the script
    let output = command
        .output()
        .map_err(|e| anyhow::anyhow!("Failed to execute script: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
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
    async fn test_script_execution() -> Result<()> {
        // Create a temporary directory for our test script
        let temp_dir = tempdir()?;

        // Create a simple test script
        let script_path = temp_dir.path().join("test_script.py");
        fs::write(
            &script_path,
            r#"
from moose_lib import task

@task
def test_task():
    return {"result": 42}
"#,
        )?;

        // Execute script
        let result = execute_script(SupportedLanguages::Python, &script_path, None).await?;

        // Verify result
        assert_eq!(result, Some(serde_json::json!({"result": 42})));

        Ok(())
    }
}
