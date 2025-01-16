use anyhow::Result;
use std::path::PathBuf;
mod python;

use crate::framework::languages::SupportedLanguages;

/// Execute a specific script
pub(crate) async fn execute_workflow(
    language: SupportedLanguages,
    script_path: &PathBuf,
    input_data: Option<serde_json::Value>,
) -> Result<Option<serde_json::Value>> {
    match language {
        SupportedLanguages::Python => python::execute_workflow(script_path, input_data).await,
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

        let result = execute_workflow(
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

        let result = execute_workflow(
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

        let result = execute_workflow(
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

    #[tokio::test]
    async fn test_child_workflow() -> Result<()> {
        let temp_dir = tempdir()?;

        // Create one sequential workflow called daily-etl
        let daily_etl_dir = temp_dir.path().join("daily-etl");
        fs::create_dir(&daily_etl_dir)?;

        // Add extract task
        fs::write(
            &daily_etl_dir.join("1.extract.py"),
            r#"
from moose_lib import task

@task
def extract():
    return {"step": "extract", "data": [1, 2, 3]}
        "#,
        )?;

        // Add transform task
        fs::write(
            &daily_etl_dir.join("2.transform.py"),
            r#"
from moose_lib import task

@task
def transform():
    return {"step": "transform", "data": [2, 4, 6]}
        "#,
        )?;

        // Create send-report directory and its child workflow marker
        fs::create_dir_all(&daily_etl_dir.join("3.send-report"))?;
        fs::write(
            &daily_etl_dir.join("3.send-report").join("child.toml"),
            r#"
# This file indicates that this directory should be treated as a child workflow
workflow = "send-report"  # References the workflow to execute
        "#,
        )?;

        // Create the send-report workflow directory
        let send_report_dir = temp_dir.path().join("send-report");
        fs::create_dir_all(&send_report_dir)?;

        // Create first step in send report workflow
        fs::write(
            &send_report_dir.join("1.prepare.py"),
            r#"
from moose_lib import task

@task
def prepare():
    return {"step": "prepare", "data": [2, 4, 6]}
        "#,
        )?;

        // Create second step in send report workflow
        fs::write(
            &send_report_dir.join("2.package.py"),
            r#"
from moose_lib import task

@task
def package():
    return {"step": "package", "data": [2, 4, 6]}
        "#,
        )?;

        // Run the workflow
        let result = execute_workflow(SupportedLanguages::Python, &daily_etl_dir, None).await?;

        // Verify the results
        assert!(result.is_some());
        let results = result.unwrap();
        let results_array = results.as_array().unwrap();

        // We expect three top-level results:
        // 1. extract result
        // 2. transform result
        // 3. send-report child workflow results (which itself is an array of two results)
        assert_eq!(results_array.len(), 3);

        // Verify extract step
        let extract_result = &results_array[0];
        assert_eq!(
            extract_result["step"],
            serde_json::Value::String("extract".to_string())
        );

        // Verify transform step
        let transform_result = &results_array[1];
        assert_eq!(
            transform_result["step"],
            serde_json::Value::String("transform".to_string())
        );

        // Verify send-report child workflow results
        let send_report_results = results_array[2].as_array().unwrap();
        assert_eq!(send_report_results.len(), 2); // prepare and package steps

        // Verify prepare step in child workflow
        let prepare_result = &send_report_results[0];
        assert_eq!(
            prepare_result["step"],
            serde_json::Value::String("prepare".to_string())
        );

        // Verify package step in child workflow
        let package_result = &send_report_results[1];
        assert_eq!(
            package_result["step"],
            serde_json::Value::String("package".to_string())
        );

        Ok(())
    }
}
