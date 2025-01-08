use std::path::{Path, PathBuf};

use super::languages::SupportedLanguages;

pub mod activity;
mod collector;
pub mod config;
pub mod executor;
pub mod generator;
pub mod orchestrator;

use crate::framework::scripts::config::WorkflowConfig;
use crate::framework::typescript::templates::TS_BASE_SCRIPT_TEMPLATE;
use crate::utilities::constants::SCRIPTS_DIR;
use crate::{
    framework::python::templates::PYTHON_BASE_SCRIPT_TEMPLATE, utilities::constants::APP_DIR,
};
use anyhow::Result;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;

// A worklfow is a collection of scripts that are executed in order
//
// Workflows are simply a file with a list of scripts within them.
//
// A workflow's name is the name of the folder that contains the scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    name: String,
    path: PathBuf,
    scripts: Vec<Script>,
    language: SupportedLanguages,
    children: Vec<Workflow>,
}

impl Workflow {
    /// Creates a workflow from a directory
    pub fn from_dir(dir: PathBuf) -> Result<Self, anyhow::Error> {
        let name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid workflow directory name"))?
            .to_string();

        let mut scripts = Vec::new();
        let mut children = Vec::new();
        let mut primary_language = None;

        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(script) = Script::from_file(path) {
                    if primary_language.is_none() {
                        primary_language = Some(script.language);
                    }
                    scripts.push(script);
                }
            } else if path.is_dir() {
                let child_toml = path.join("child.toml");
                if child_toml.exists() {
                    // Use the directory name as the referenced workflow name
                    if let Some(workflow_name) = path.file_name().and_then(|n| n.to_str()) {
                        let workflow_name =
                            workflow_name.split('.').last().unwrap_or(workflow_name);
                        if let Some(root_dir) = dir.parent() {
                            let referenced_workflow_path = root_dir.join(workflow_name);
                            if let Ok(child_workflow) = Self::from_dir(referenced_workflow_path) {
                                children.push(child_workflow);
                            }
                        }
                    }
                } else {
                    // Check for parallel execution directory
                    for file in std::fs::read_dir(path)? {
                        if let Ok(script) = Script::from_file(file?.path()) {
                            scripts.push(script);
                        }
                    }
                }
            }
        }

        scripts.sort_by_key(|script| script.order);

        Ok(Self {
            name,
            path: dir,
            scripts,
            language: primary_language.ok_or_else(|| anyhow::anyhow!("No valid scripts found"))?,
            children,
        })
    }

    /// Initialize a new workflow with a list of scripts
    pub fn init(
        name: &str,
        scripts: &[String],
        language: SupportedLanguages,
    ) -> Result<(), anyhow::Error> {
        let workflow_dir = std::env::current_dir()?
            .join(APP_DIR)
            .join(SCRIPTS_DIR)
            .join(name);
        std::fs::create_dir_all(&workflow_dir)?;

        // Create config.toml with workflow name and steps
        let config = if scripts.is_empty() {
            WorkflowConfig::new(name.to_string())
        } else {
            WorkflowConfig::with_steps(name.to_string(), scripts.to_vec())
        };
        config.save(workflow_dir.join("config.toml"))?;

        // Create each script with an order number
        for (index, script_name) in scripts.iter().enumerate() {
            Script::init(
                script_name,
                Some((index + 1) as u32),
                language,
                &workflow_dir,
            )?;
        }

        Ok(())
    }

    /// Add a new script to an existing workflow
    pub fn add_script(workflow_name: &str, script_name: &str) -> Result<(), anyhow::Error> {
        let workflow_dir = Path::new(SCRIPTS_DIR).join(workflow_name);
        if !workflow_dir.exists() {
            return Err(anyhow::anyhow!("Workflow {} does not exist", workflow_name));
        }

        // Find the highest order number in the workflow
        let mut max_order = 0;
        for entry in std::fs::read_dir(&workflow_dir)? {
            let entry = entry?;
            if let Ok(script) = Script::from_file(entry.path()) {
                if let Some(order) = script.order {
                    max_order = max_order.max(order);
                }
            }
        }

        // Infer the language from existing scripts
        let workflow = Self::from_dir(workflow_dir.clone())?;

        Script::init(
            script_name,
            Some(max_order + 1),
            workflow.language,
            &workflow_dir,
        )?;

        Ok(())
    }

    /// Start the workflow execution locally
    pub async fn start(&self) -> Result<Option<serde_json::Value>, anyhow::Error> {
        executor::execute_workflow(self.language, &self.path, None).await
    }

    /// Check if this workflow is currently running
    pub async fn is_running(&self) -> Result<bool, anyhow::Error> {
        orchestrator::is_workflow_running(&self.name)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to check workflow status: {}", e))
    }

    /// Stop the workflow execution
    pub async fn stop(&self) -> Result<(), anyhow::Error> {
        orchestrator::stop_workflow(&self.name)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to stop workflow: {}", e))
    }
}

// A collection of workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflows {
    workflows: Vec<Workflow>,
}

impl Workflows {
    pub fn from_dir(dir: PathBuf) -> Result<Self, anyhow::Error> {
        let mut workflows = Vec::new();

        // Read all entries in the directory
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            // If it's a directory, try to create a workflow from it
            if path.is_dir() {
                if let Ok(workflow) = Workflow::from_dir(path) {
                    workflows.push(workflow);
                }
            }
        }

        Ok(Self { workflows })
    }

    /// Get all workflows
    pub fn get_defined_workflows(&self) -> &Vec<Workflow> {
        &self.workflows
    }

    /// Get a specific workflow by name
    pub fn get_defined_workflow(&self, name: &str) -> Option<&Workflow> {
        self.workflows.iter().find(|w| w.name == name)
    }

    /// Get all running workflows
    pub async fn get_running_workflows(&self) -> Result<Vec<Workflow>, anyhow::Error> {
        let mut running_workflows = Vec::new();

        for workflow in &self.workflows {
            match orchestrator::is_workflow_running(&workflow.name).await {
                Ok(true) => running_workflows.push(workflow.clone()),
                Ok(false) | Err(_) => continue, // Skip workflows that aren't found or have errors
            }
        }

        Ok(running_workflows)
    }
}

// A script is a file that contains a temporal activity.
//
// Scripts have a specific naming convention
// <order>.<script-name>.<language-ext>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Script {
    path: PathBuf,
    order: Option<u32>,
    name: String,
    language: SupportedLanguages,
}

impl Script {
    /// Creates a script from a file path, supporting two naming conventions:
    /// - Ordered scripts: "<order>.<script-name>.<language-ext>" (e.g. "1.extract.py")
    /// - Unordered scripts: "<script-name>.<language-ext>" (e.g. "extract.py")
    fn from_file(path: PathBuf) -> Result<Self, anyhow::Error> {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

        let parts: Vec<&str> = file_name.split('.').collect();
        if parts.len() < 2 {
            return Err(anyhow::anyhow!(
                "Invalid script file format. Expected: <name>.<ext> or <order>.<name>.<ext>"
            ));
        }

        let (order, name) = if parts.len() >= 3 {
            // Try to parse as ordered script
            match parts[0].parse::<u32>() {
                Ok(num) => (Some(num), parts[1].to_string()),
                Err(_) => (None, parts[0].to_string()),
            }
        } else {
            // Unordered script
            (None, parts[0].to_string())
        };

        let language = infer_language(path.clone())?;

        Ok(Self {
            path,
            order,
            name,
            language,
        })
    }

    /// Creates a new script file with boilerplate code based on the language
    pub fn init(
        name: &str,
        order: Option<u32>,
        language: SupportedLanguages,
        dir: &Path,
    ) -> Result<(), anyhow::Error> {
        let file_name = match order {
            Some(num) => format!("{}.{}.{}", num, name, language.extension()),
            None => format!("{}.{}", name, language.extension()),
        };

        let handlebars = Handlebars::new();
        let template = match language {
            SupportedLanguages::Python => PYTHON_BASE_SCRIPT_TEMPLATE,
            SupportedLanguages::Typescript => TS_BASE_SCRIPT_TEMPLATE,
        };

        let content = handlebars.render_template(
            template,
            &json!({
                "name": name
            }),
        )?;

        std::fs::write(dir.join(file_name), content)?;
        Ok(())
    }
}

/// Infers the supported language from the file extension of a file path
pub fn infer_language(path: PathBuf) -> Result<SupportedLanguages> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("py") => Ok(SupportedLanguages::Python),
        Some("ts") | Some("js") => Ok(SupportedLanguages::Typescript),
        _ => Err(anyhow::anyhow!("Unsupported language")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Create a unique workflow ID
    fn unique_workflow_id() -> String {
        format!("test-workflow-{}", uuid::Uuid::new_v4())
    }

    #[test]
    fn test_infer_language() {
        assert_eq!(
            infer_language(PathBuf::from("test.py")).unwrap(),
            SupportedLanguages::Python
        );
        assert_eq!(
            infer_language(PathBuf::from("test.ts")).unwrap(),
            SupportedLanguages::Typescript
        );
    }

    #[test]
    fn test_ordered_script_from_file() {
        let script = Script::from_file(PathBuf::from("1.extract.py")).unwrap();
        assert_eq!(script.order, Some(1));
        assert_eq!(script.name, "extract");
        assert_eq!(script.language, SupportedLanguages::Python);
    }

    #[test]
    fn test_unordered_script_from_file() {
        let script = Script::from_file(PathBuf::from("extract.py")).unwrap();
        assert_eq!(script.order, None);
        assert_eq!(script.name, "extract");
        assert_eq!(script.language, SupportedLanguages::Python);
    }

    #[test]
    fn test_invalid_script_name() {
        // Test a file with no extension
        assert!(Script::from_file(PathBuf::from("invalid")).is_err());

        // Test a file with unsupported extension
        assert!(Script::from_file(PathBuf::from("script.xyz")).is_err());

        // Test a completely empty filename
        assert!(Script::from_file(PathBuf::from("")).is_err());
    }

    #[test]
    fn test_script_init() {
        let temp_dir = tempdir().unwrap();
        Script::init(
            "extract",
            Some(1),
            SupportedLanguages::Python,
            &temp_dir.path(),
        )
        .unwrap();
        assert!(temp_dir.path().join("1.extract.py").exists());
        // Make sure the file contains @task
        let content = std::fs::read_to_string(temp_dir.path().join("1.extract.py")).unwrap();
        assert!(content.contains("@task"));
    }

    #[test]
    fn test_workflow_init() {
        let temp_dir = tempdir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();
        std::fs::create_dir(SCRIPTS_DIR).unwrap();

        Workflow::init(
            "daily-etl",
            &["extract".to_string(), "transform".to_string()],
            SupportedLanguages::Python,
        )
        .unwrap();

        assert!(temp_dir.path().join(SCRIPTS_DIR).join("daily-etl").exists());
        assert!(temp_dir
            .path()
            .join(SCRIPTS_DIR)
            .join("daily-etl/1.extract.py")
            .exists());
        assert!(temp_dir
            .path()
            .join(SCRIPTS_DIR)
            .join("daily-etl/2.transform.py")
            .exists());
    }

    #[test]
    fn test_workflow_from_dir() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;

        // Create main workflow
        let daily_etl_dir = temp_dir.path().join("daily-etl");
        fs::create_dir(&daily_etl_dir)?;

        // Create main scripts
        fs::write(daily_etl_dir.join("1.extract.py"), "# Extract")?;
        fs::write(daily_etl_dir.join("2.transform.py"), "# Transform")?;

        // Create parallel execution directory
        let parallel_dir = daily_etl_dir.join("3.parallel");
        fs::create_dir(&parallel_dir)?;
        fs::write(parallel_dir.join("process-a.py"), "# Parallel A")?;
        fs::write(parallel_dir.join("process-b.py"), "# Parallel B")?;

        // Create reference to child workflow
        let send_report_ref_dir = daily_etl_dir.join("5.send-report");
        fs::create_dir(&send_report_ref_dir)?;
        fs::write(
            send_report_ref_dir.join("child.toml"),
            r#"workflow = "send-report""#,
        )?;

        // Create separate send-report workflow at root level
        let send_report_dir = temp_dir.path().join("send-report");
        fs::create_dir(&send_report_dir)?;
        fs::write(send_report_dir.join("1.prepare.py"), "# Prepare report")?;
        fs::write(send_report_dir.join("2.package.py"), "# Package report")?;
        fs::write(send_report_dir.join("3.send.py"), "# Send report")?;

        let workflow = Workflow::from_dir(daily_etl_dir)?;

        // Test main workflow
        assert_eq!(workflow.name, "daily-etl");
        assert_eq!(workflow.language, SupportedLanguages::Python);

        // Verify all scripts are present (without checking order)
        let script_names: Vec<_> = workflow.scripts.iter().map(|s| &s.name).collect();
        assert!(script_names.contains(&&"extract".to_string()));
        assert!(script_names.contains(&&"transform".to_string()));
        assert!(script_names.contains(&&"process-a".to_string()));
        assert_eq!(workflow.scripts.len(), 4);

        // Verify child workflow
        assert_eq!(workflow.children.len(), 1);
        let child_workflow = &workflow.children[0];
        assert_eq!(child_workflow.name, "send-report");

        let child_script_names: Vec<_> = child_workflow.scripts.iter().map(|s| &s.name).collect();
        assert!(child_script_names.contains(&&"prepare".to_string()));
        assert!(child_script_names.contains(&&"package".to_string()));
        assert!(child_script_names.contains(&&"send".to_string()));
        assert_eq!(child_workflow.scripts.len(), 3);

        Ok(())
    }

    #[test]
    fn test_workflows_from_dir() -> Result<(), anyhow::Error> {
        let temp_dir = tempdir()?;
        let scripts_dir = temp_dir.path().join("scripts");
        std::fs::create_dir_all(&scripts_dir)?;

        // Create two workflow directories
        let workflow1_dir = scripts_dir.join("workflow1");
        let workflow2_dir = scripts_dir.join("workflow2");
        std::fs::create_dir_all(&workflow1_dir)?;
        std::fs::create_dir_all(&workflow2_dir)?;

        // Add a script to each workflow
        std::fs::write(
            workflow1_dir.join("1.test.py"),
            r#"
from moose_lib import task

@task
def test():
    return {"step": "test1"}
            "#,
        )?;

        std::fs::write(
            workflow2_dir.join("1.test.py"),
            r#"
from moose_lib import task

@task
def test():
    return {"step": "test2"}
            "#,
        )?;

        let workflows = Workflows::from_dir(scripts_dir)?;
        assert_eq!(workflows.get_defined_workflows().len(), 2);

        // Test getting specific workflow
        let workflow1 = workflows.get_defined_workflow("workflow1");
        assert!(workflow1.is_some());
        assert_eq!(workflow1.unwrap().name, "workflow1");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_running_workflows() -> Result<(), anyhow::Error> {
        let workflow_id = unique_workflow_id();
        let temp_dir = tempdir()?;
        let scripts_dir = temp_dir.path().join("scripts");
        std::fs::create_dir_all(&scripts_dir)?;

        // Create a workflow directory
        let workflow_dir = scripts_dir.join(&workflow_id);
        std::fs::create_dir_all(&workflow_dir)?;

        // Add a script that runs long enough
        std::fs::write(
            workflow_dir.join("1.test.py"),
            r#"
from moose_lib import task
import time

@task
def test():
    time.sleep(5)  # Longer sleep to ensure registration
    return {"step": "test"}
        "#,
        )?;

        let workflows = Workflows::from_dir(scripts_dir)?;

        // Initially should have no running workflows
        let initial_running = workflows.get_running_workflows().await?;
        assert!(
            initial_running.is_empty(),
            "Expected no running workflows initially"
        );

        // Start the workflow
        if let Some(workflow) = workflows.get_defined_workflow(&workflow_id).cloned() {
            let workflow_clone = workflow.clone();
            tokio::spawn(async move {
                let _ = workflow_clone.start().await;
            });

            // Give it a moment to start
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Check if it's running
            let is_running = workflow.is_running().await?;
            assert!(is_running, "Workflow should be running");

            // Stop the workflow
            let _ = workflow.stop().await;

            // Check if it's stopped
            let is_stopped = workflow.is_running().await?;
            assert!(!is_stopped, "Workflow should be stopped");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_start_workflow() -> Result<(), anyhow::Error> {
        let temp_dir = tempdir()?;
        let scripts_dir = temp_dir.path().join(SCRIPTS_DIR);
        std::fs::create_dir_all(&scripts_dir)?; // Create all directories in path
        std::env::set_current_dir(&temp_dir)?;

        let workflow_id = unique_workflow_id();

        // Initialize a workflow with a simple script
        Workflow::init(
            &workflow_id,
            &["simple".to_string()],
            SupportedLanguages::Python,
        )?;

        // Load the workflow and start it
        let workflow = Workflow::from_dir(scripts_dir.join(&workflow_id))?;
        let result = workflow.start().await?;

        assert!(result.is_some(), "Workflow should return a result");
        Ok(())
    }

    #[tokio::test]
    async fn test_workflow_is_running() -> Result<(), anyhow::Error> {
        let temp_dir = tempdir()?;
        let scripts_dir = temp_dir.path().join(SCRIPTS_DIR);
        std::fs::create_dir_all(&scripts_dir)?;
        std::env::set_current_dir(&temp_dir)?;

        let workflow_id = unique_workflow_id();

        // Initialize a workflow with a script that sleeps
        Workflow::init(
            &workflow_id,
            &["long-running".to_string()],
            SupportedLanguages::Python,
        )?;

        // Modify the script to include a sleep
        let script_path = scripts_dir.join(&workflow_id).join("1.long-running.py");
        std::fs::write(
            script_path,
            r#"
from moose_lib import task
import time

@task
def long_running():
    time.sleep(5)  # Longer sleep to ensure we can check status
    return {"status": "completed"}
"#,
        )?;

        // Load and start the workflow
        let workflow = Workflow::from_dir(scripts_dir.join(&workflow_id))?;

        let workflow_clone = workflow.clone();
        tokio::spawn(async move {
            let _ = workflow_clone.start().await;
        });

        // Wait for workflow to start with retries
        let mut is_running = false;
        for _ in 0..10 {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            match workflow.is_running().await {
                Ok(true) => {
                    is_running = true;
                    break;
                }
                _ => continue,
            }
        }
        assert!(is_running, "Workflow should be running");

        // Stop the workflow
        workflow.stop().await?;

        // Wait for workflow to stop with retries
        let mut is_stopped = false;
        for _ in 0..10 {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            match workflow.is_running().await {
                Ok(false) => {
                    is_stopped = true;
                    break;
                }
                _ => continue,
            }
        }
        assert!(is_stopped, "Workflow should be stopped");

        Ok(())
    }
}
