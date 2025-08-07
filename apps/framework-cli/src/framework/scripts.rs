use std::path::{Path, PathBuf};

use super::languages::SupportedLanguages;

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

    pub fn from_user_code(
        name: String,
        language: SupportedLanguages,
        retries: Option<u32>,
        timeout: Option<String>,
        schedule: Option<String>,
    ) -> Result<Self, anyhow::Error> {
        let config = WorkflowConfig::with_overrides(name.clone(), retries, timeout, schedule);

        Ok(Self {
            name: name.clone(),
            path: PathBuf::from(name.clone()),
            scripts: Vec::new(),
            config,
            language,
            children: Vec::new(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn config(&self) -> &WorkflowConfig {
        &self.config
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
            &self.config,
            &self.path,
            input,
        )
        .await?)
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
