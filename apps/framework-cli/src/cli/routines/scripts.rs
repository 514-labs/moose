use anyhow::Result;
use std::path::PathBuf;

use crate::cli::display::Message;
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::framework::languages::SupportedLanguages;
use crate::framework::scripts::Workflow;
use crate::utilities::constants::{APP_DIR, SCRIPTS_DIR};

pub async fn init_workflow(
    name: &str,
    language: SupportedLanguages,
    steps: Option<String>,
    step: Option<Vec<String>>,
) -> Result<RoutineSuccess, RoutineFailure> {
    // Convert steps string to vector if present
    let step_vec = if let Some(steps_str) = steps {
        steps_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        step.unwrap_or_default()
    };

    // Initialize the workflow using the existing Workflow::init method
    Workflow::init(name, &step_vec, language).map_err(|e| {
        RoutineFailure::new(
            Message {
                action: "Workflow Init Failed".to_string(),
                details: format!("Could not initialize workflow '{}': {}", name, e),
            },
            e,
        )
    })?;

    // Return success with helpful next steps
    Ok(RoutineSuccess::success(Message {
        action: "Created".to_string(),
        details: format!(
            "Workflow '{}' initialized successfully\n\nNext Steps:\n1. cd {}/{}\n2. Edit your workflow steps\n3. Run with: moose-cli workflow run {}",
            name, SCRIPTS_DIR, name, name
        ),
    }))
}

pub async fn run_workflow(name: &str) -> Result<RoutineSuccess, RoutineFailure> {
    // Workflow directory is in app/scripts
    let workflow_dir = PathBuf::from(APP_DIR).join(SCRIPTS_DIR).join(name);
    let workflow = Workflow::from_dir(workflow_dir.clone()).map_err(|e| {
        RoutineFailure::new(
            Message {
                action: "Workflow Start Failed".to_string(),
                details: format!(
                    "Could not create workflow '{}' from directory {}: {}",
                    name,
                    workflow_dir.display(),
                    e
                ),
            },
            e,
        )
    })?;
    workflow.start().await.map_err(|e| {
        RoutineFailure::new(
            Message {
                action: "Workflow Start Failed".to_string(),
                details: format!("Could not start workflow '{}': {}", name, e),
            },
            e,
        )
    })?;
    Ok(RoutineSuccess::success(Message {
        action: "Workflow Started".to_string(),
        details: format!("Workflow '{}' started successfully", name),
    }))
}

#[cfg(test)]
mod tests {
    use crate::utilities::constants::APP_DIR;

    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        std::fs::create_dir_all(SCRIPTS_DIR).unwrap();
        temp_dir
    }

    #[tokio::test]
    async fn test_workflow_init_basic() {
        let temp_dir = setup();

        let result = init_workflow("daily-etl", SupportedLanguages::Python, None, None)
            .await
            .unwrap();

        assert!(result.message.details.contains("daily-etl"));

        let workflow_dir = temp_dir
            .path()
            .join(APP_DIR)
            .join(SCRIPTS_DIR)
            .join("daily-etl");
        assert!(
            workflow_dir.exists(),
            "Workflow directory should be created in app/scripts"
        );

        let config_path = workflow_dir.join("config.toml");
        assert!(config_path.exists(), "config.toml should be created");
    }

    #[tokio::test]
    async fn test_workflow_init_with_steps() {
        let temp_dir = setup();

        let result = init_workflow(
            "daily-etl",
            SupportedLanguages::Python,
            Some("extract,transform,load".to_string()),
            None,
        )
        .await
        .unwrap();

        assert!(result.message.details.contains("daily-etl"));

        let workflow_dir = temp_dir
            .path()
            .join(APP_DIR)
            .join(SCRIPTS_DIR)
            .join("daily-etl");

        for (i, step) in ["extract", "transform", "load"].iter().enumerate() {
            let file_path = workflow_dir.join(format!("{}.{}.py", i + 1, step));
            assert!(file_path.exists(), "Step file {} should exist", step);

            let content = fs::read_to_string(&file_path).unwrap();
            assert!(content.contains("@task"));
        }
    }

    #[tokio::test]
    async fn test_workflow_init_failure() {
        let temp_dir = setup();

        // Create a file where the workflow directory should be to cause a failure
        fs::create_dir_all(temp_dir.path().join(APP_DIR).join(SCRIPTS_DIR)).unwrap();
        fs::write(
            temp_dir
                .path()
                .join(APP_DIR)
                .join(SCRIPTS_DIR)
                .join("daily-etl"),
            "",
        )
        .unwrap();

        let result = init_workflow("daily-etl", SupportedLanguages::Python, None, None).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message.action, "Workflow Init Failed");
    }

    #[tokio::test]
    async fn test_run_workflow() {
        let temp_dir = setup();

        // First initialize a workflow
        let _ = init_workflow(
            "test-workflow",
            SupportedLanguages::Python,
            Some("extract,transform,load".to_string()),
            None,
        )
        .await
        .unwrap();

        // Verify workflow exists
        let workflow_dir = temp_dir
            .path()
            .join(APP_DIR)
            .join(SCRIPTS_DIR)
            .join("test-workflow");
        assert!(workflow_dir.exists(), "Workflow directory should exist");

        // Run the workflow
        let result = run_workflow("test-workflow").await;
        println!("Result: {:?}", result);
        assert!(result.is_ok(), "Workflow should run successfully");

        let success = result.unwrap();
        assert!(success.message.details.contains("started successfully"));
    }
}
