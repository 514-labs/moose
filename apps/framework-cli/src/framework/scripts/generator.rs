use anyhow::Result;

use crate::framework::{
    languages::SupportedLanguages,
    scripts::{config::WorkflowConfig, Workflow},
};

pub fn handle_workflow_init(
    name: &str,
    language: SupportedLanguages,
    steps: Option<String>,
    step: Option<Vec<String>>,
) -> Result<()> {
    // Convert steps string to vector if present
    let step_vec = if let Some(steps_str) = steps {
        Some(steps_str.split(',').map(|s| s.trim().to_string()).collect())
    } else {
        step
    };

    // Initialize workflow with steps if provided
    if let Some(steps) = step_vec {
        Workflow::init(name, &steps, language)?;

        // Create and save workflow config
        let config = WorkflowConfig::with_steps(name.to_string(), steps);
        let config_path = std::env::current_dir()?
            .join("workflows")
            .join(name)
            .join("config.toml");
        config.save(config_path)?;
    } else {
        // Initialize empty workflow
        Workflow::init(name, &[], language)?;

        // Create and save basic workflow config
        let config = WorkflowConfig::new(name.to_string());
        let config_path = std::env::current_dir()?
            .join("workflows")
            .join(name)
            .join("config.toml");
        config.save(config_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        temp_dir
    }

    #[test]
    fn test_handle_workflow_init_basic() {
        let temp_dir = setup();

        // Test basic workflow initialization
        handle_workflow_init("daily-etl", SupportedLanguages::Python, None, None).unwrap();

        // Check directory structure
        let workflow_dir = temp_dir.path().join("workflows").join("daily-etl");
        assert!(
            workflow_dir.exists(),
            "Workflow directory should be created"
        );
        assert!(workflow_dir.is_dir(), "Workflow path should be a directory");

        // Check config.toml
        let config_path = workflow_dir.join("config.toml");
        assert!(config_path.exists(), "config.toml should be created");

        let config_content = fs::read_to_string(config_path).unwrap();
        assert!(config_content.contains("name = 'daily-etl'"));
        assert!(config_content.contains("[data.processing]"));
        assert!(config_content.contains("[resources]"));
    }

    #[test]
    fn test_handle_workflow_init_with_steps_string() {
        let temp_dir = setup();

        // Test workflow initialization with steps string
        handle_workflow_init(
            "daily-etl",
            SupportedLanguages::Python,
            Some("extract,transform,load".to_string()),
            None,
        )
        .unwrap();

        let workflow_dir = temp_dir.path().join("workflows").join("daily-etl");

        // Check step files
        let expected_files = ["1.extract.py", "2.transform.py", "3.load.py"];
        for file in expected_files.iter() {
            let file_path = workflow_dir.join(file);
            assert!(file_path.exists(), "Step file {} should exist", file);

            let content = fs::read_to_string(&file_path).unwrap();
            assert!(
                content.contains("@task"),
                "Step file should contain @task decorator"
            );
            assert!(
                content.contains("def main"),
                "Step file should contain main function"
            );
        }

        // Check config.toml contains steps
        let config_path = workflow_dir.join("config.toml");
        let config_content = fs::read_to_string(config_path).unwrap();
        assert!(config_content.contains("name = 'daily-etl'"));
        assert!(config_content.contains("[steps]"));
        assert!(config_content.contains("steps = ['extract', 'transform', 'load']"));
    }

    #[test]
    fn test_handle_workflow_init_with_step_vec() {
        let temp_dir = setup();

        // Test workflow initialization with step vector
        handle_workflow_init(
            "daily-etl",
            SupportedLanguages::Python,
            None,
            Some(vec![
                "extract".to_string(),
                "transform".to_string(),
                "load".to_string(),
            ]),
        )
        .unwrap();

        let workflow_dir = temp_dir.path().join("workflows").join("daily-etl");

        // Check step files
        let expected_files = ["1.extract.py", "2.transform.py", "3.load.py"];
        for file in expected_files.iter() {
            let file_path = workflow_dir.join(file);
            assert!(file_path.exists(), "Step file {} should exist", file);

            let content = fs::read_to_string(&file_path).unwrap();
            assert!(
                content.contains("@task"),
                "Step file should contain @task decorator"
            );
            assert!(
                content.contains("def main"),
                "Step file should contain main function"
            );
        }

        // Check config.toml contains steps
        let config_path = workflow_dir.join("config.toml");
        let config_content = fs::read_to_string(config_path).unwrap();
        assert!(config_content.contains("name = 'daily-etl'"));
        assert!(config_content.contains("[steps]"));
        assert!(config_content.contains("steps = ['extract', 'transform', 'load']"));
    }
}
