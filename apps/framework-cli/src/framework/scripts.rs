use std::path::{Path, PathBuf};

use super::languages::SupportedLanguages;

pub mod collector;
pub mod config;
pub mod executor;
pub mod utils;

use crate::framework::python::templates::PYTHON_BASE_SCRIPT_TEMPLATE;
use crate::framework::scripts::config::WorkflowConfig;
use crate::framework::typescript::templates::TS_BASE_SCRIPT_TEMPLATE;
use crate::infrastructure::orchestration::temporal::TemporalConfig;
use crate::project::Project;
use crate::utilities::constants::SCRIPTS_DIR;
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
    config: WorkflowConfig,
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
        let mut config = None;

        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if path.file_name().and_then(|n| n.to_str()) == Some("config.toml") {
                    config = Some(WorkflowConfig::from_file(path)?);
                } else if let Ok(script) = Script::from_file(path) {
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
                        let workflow_name = workflow_name
                            .split('.')
                            .next_back()
                            .unwrap_or(workflow_name);
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
            config: config.ok_or_else(|| anyhow::anyhow!("No config.toml found"))?,
            language: primary_language.ok_or_else(|| anyhow::anyhow!("No valid scripts found"))?,
            children,
        })
    }

    /// Initialize a new workflow with a list of scripts
    pub fn init(project: &Project, name: &str, scripts: &[String]) -> Result<(), anyhow::Error> {
        let scripts_dir = project.scripts_dir();
        let workflow_dir = scripts_dir.join(name);

        std::fs::create_dir_all(&workflow_dir)?;

        // Create config.toml with workflow name and tasks
        let config = if scripts.is_empty() {
            WorkflowConfig::new(name.to_string())
        } else {
            WorkflowConfig::with_tasks(name.to_string(), scripts.to_vec())
        };
        config.save(workflow_dir.join("config.toml"))?;

        // Create each script with an order number
        for (index, script_name) in scripts.iter().enumerate() {
            Script::init(
                script_name,
                Some((index + 1) as u32),
                project.language,
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
    pub async fn start(
        &self,
        temporal_config: &TemporalConfig,
        input: Option<String>,
    ) -> Result<String, anyhow::Error> {
        Ok(executor::execute_workflow(
            temporal_config,
            self.language,
            &self.name,
            &self.path,
            input,
        )
        .await?)
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
    use crate::utilities::constants::APP_DIR;

    use super::*;
    use std::fs;
    use tempfile::tempdir;

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
        // Make sure the file contains @task()
        let content = std::fs::read_to_string(temp_dir.path().join("1.extract.py")).unwrap();
        assert!(content.contains("@task()"));
    }

    #[test]
    fn test_workflow_init() -> Result<(), anyhow::Error> {
        let temp_dir = tempdir()?;

        let project = Project::new(
            temp_dir.path(),
            "project-name".to_string(),
            SupportedLanguages::Python,
        );

        Workflow::init(
            &project,
            "daily-etl",
            &["extract".to_string(), "transform".to_string()],
        )?;

        assert!(temp_dir
            .path()
            .join(APP_DIR)
            .join(SCRIPTS_DIR)
            .join("daily-etl")
            .exists());

        assert!(temp_dir
            .path()
            .join(APP_DIR)
            .join(SCRIPTS_DIR)
            .join("daily-etl/1.extract.py")
            .exists());

        assert!(temp_dir
            .path()
            .join(APP_DIR)
            .join(SCRIPTS_DIR)
            .join("daily-etl/2.transform.py")
            .exists());

        Ok(())
    }

    #[test]
    fn test_workflow_from_dir() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;

        // Create main workflow
        let daily_etl_dir = temp_dir.path().join("daily-etl");
        fs::create_dir(&daily_etl_dir)?;

        let daily_etl_config_path = daily_etl_dir.join("config.toml");
        let daily_etl_config = WorkflowConfig::new("daily-etl".to_string());
        daily_etl_config.save(daily_etl_config_path)?;

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
        let send_report_config_path = send_report_dir.join("config.toml");
        let send_report_config = WorkflowConfig::new("send-report".to_string());
        send_report_config.save(send_report_config_path)?;
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

@task()
def test():
    return {"step": "test1"}
            "#,
        )?;

        std::fs::write(
            workflow2_dir.join("1.test.py"),
            r#"
from moose_lib import task

@task()
def test():
    return {"step": "test2"}
            "#,
        )?;

        let workflow1_config_path = workflow1_dir.join("config.toml");
        let workflow1_config = WorkflowConfig::new("workflow1".to_string());
        workflow1_config.save(workflow1_config_path)?;

        let workflow2_config_path = workflow2_dir.join("config.toml");
        let workflow2_config = WorkflowConfig::new("workflow2".to_string());
        workflow2_config.save(workflow2_config_path)?;

        let workflows = Workflows::from_dir(scripts_dir)?;
        assert_eq!(workflows.get_defined_workflows().len(), 2);

        // Test getting specific workflow
        let workflow1 = workflows.get_defined_workflow("workflow1");
        assert!(workflow1.is_some());
        assert_eq!(workflow1.unwrap().name, "workflow1");

        Ok(())
    }
}
