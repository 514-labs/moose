use anyhow::Result;

use crate::cli::display::Message;
use crate::cli::routines::{RoutineFailure, RoutineSuccess};
use crate::framework::scripts::Workflow;
use crate::project::Project;
use crate::utilities::constants::SCRIPTS_DIR;

pub async fn init_workflow(
    project: &Project,
    name: &str,
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
    Workflow::init(project, name, &step_vec).map_err(|e| {
        RoutineFailure::new(
            Message {
                action: "Workflow Init Failed".to_string(),
                details: format!("Could not initialize workflow '{}': {}", project.name(), e),
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

pub async fn run_workflow(project: &Project, name: &str) -> Result<RoutineSuccess, RoutineFailure> {
    // Workflow directory is in app/scripts
    let workflow_dir = project.scripts_dir().join(name);

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
    use crate::framework::languages::SupportedLanguages;
    use crate::project::Project;

    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> Project {
        let temp_dir = TempDir::new().unwrap();
        let project = Project::new(
            temp_dir.path(),
            "project-name".to_string(),
            SupportedLanguages::Python,
        );
        project
    }

    #[tokio::test]
    async fn test_workflow_init_basic() {
        let project = setup();

        let result = init_workflow(&project, "daily-etl", None, None)
            .await
            .unwrap();

        assert!(result.message.details.contains("daily-etl"));

        let workflow_dir = project.app_dir().join(SCRIPTS_DIR).join("daily-etl");

        assert!(
            workflow_dir.exists(),
            "Workflow directory should be created in app/scripts"
        );

        let config_path = workflow_dir.join("config.toml");
        assert!(config_path.exists(), "config.toml should be created");
    }

    #[tokio::test]
    async fn test_workflow_init_with_steps() {
        let project = setup();

        let result = init_workflow(
            &project,
            "daily-etl",
            Some("extract,transform,load".to_string()),
            None,
        )
        .await
        .unwrap();

        assert!(result.message.details.contains("daily-etl"));

        let workflow_dir = project.app_dir().join(SCRIPTS_DIR).join("daily-etl");

        for (i, step) in ["extract", "transform", "load"].iter().enumerate() {
            let file_path = workflow_dir.join(format!("{}.{}.py", i + 1, step));
            assert!(file_path.exists(), "Step file {} should exist", step);

            let content = fs::read_to_string(&file_path).unwrap();
            assert!(content.contains("@task"));
        }
    }

    #[tokio::test]
    async fn test_workflow_init_failure() {
        let project = setup();

        // Create a file where the workflow directory should be to cause a failure
        fs::create_dir_all(project.app_dir().join(SCRIPTS_DIR)).unwrap();
        fs::write(project.app_dir().join(SCRIPTS_DIR).join("daily-etl"), "").unwrap();

        let result = init_workflow(&project, "daily-etl", None, None).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message.action, "Workflow Init Failed");
    }

    // Test is ignored because it requires temporal as a dependency
    #[ignore]
    #[tokio::test]
    async fn test_run_workflow() {
        let project = setup();

        const WORKFLOW_NAME: &str = "daily-etl";

        // First initialize a workflow
        let _ = init_workflow(
            &project,
            WORKFLOW_NAME,
            Some("extract,transform,load".to_string()),
            None,
        )
        .await
        .unwrap();

        // Verify workflow exists
        let workflow_dir = project.app_dir().join(SCRIPTS_DIR).join(WORKFLOW_NAME);
        assert!(workflow_dir.exists(), "Workflow directory should exist");

        // Run the workflow
        let result = run_workflow(&project, WORKFLOW_NAME).await;
        println!("Result: {:?}", result);
        assert!(result.is_ok(), "Workflow should run successfully");

        let success = result.unwrap();
        assert!(success.message.details.contains("started successfully"));
    }
}
