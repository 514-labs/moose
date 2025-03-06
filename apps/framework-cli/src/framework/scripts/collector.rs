use crate::framework::languages::SupportedLanguages;
use std::{collections::HashMap, fs, path::PathBuf};

use super::{Script, Workflow};

/// A trait that defines the interface for a collector.
///
/// The trait is generic over `Item` (like `Script` or `Workflow`) so it can be
/// implemented by both script collectors and workflow collectors.
pub trait Collector {
    type Item;

    /// Creates a fresh collector instance.
    ///
    /// This is used as a skeleton implementationm, even through dynamic dispatch is not
    /// leveraged.
    #[allow(dead_code)]
    fn new() -> Self;

    /// Traverses the provided path to collect items (scripts, workflows, etc.).
    fn collect(
        &mut self,
        path: PathBuf,
    ) -> Result<&HashMap<SupportedLanguages, Vec<Self::Item>>, std::io::Error>;

    /// Returns a reference to the internal registry of items organized by language.
    /// This is used as a skeleton implementationm, even through dynamic dispatch is not
    /// leveraged.
    #[allow(dead_code)]
    fn items(&self) -> &std::collections::HashMap<SupportedLanguages, Vec<Self::Item>>;
}

/// A collector for scripts. It implements the Collector trait
/// and uses Script::from_file for script creation.
pub struct ScriptCollector {
    scripts_by_language: HashMap<SupportedLanguages, Vec<Script>>,
}

impl ScriptCollector {
    /// Creates a new `ScriptCollector`.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            scripts_by_language: HashMap::new(),
        }
    }
}

impl Collector for ScriptCollector {
    type Item = Script;

    fn new() -> Self {
        Self::new()
    }

    /// Traverses `path` to collect scripts into an internal map keyed by language.
    ///
    /// # Errors
    ///
    /// Returns an I/O Error if reading the directory fails.
    fn collect(
        &mut self,
        path: PathBuf,
    ) -> Result<&HashMap<SupportedLanguages, Vec<Self::Item>>, std::io::Error> {
        let entries = fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                self.collect(entry.path())?;
            } else if metadata.is_file() {
                if let Ok(script) = Script::from_file(entry.path()) {
                    self.scripts_by_language
                        .entry(script.language)
                        .or_default()
                        .push(script);
                }
            }
        }

        Ok(&self.scripts_by_language)
    }

    /// Returns a reference to the internal registry of items organized by language.
    fn items(&self) -> &HashMap<SupportedLanguages, Vec<Self::Item>> {
        &self.scripts_by_language
    }
}

/// A collector for workflows, organized by language.
pub struct WorkflowCollector {
    workflows_by_language: HashMap<SupportedLanguages, Vec<Workflow>>,
}

impl WorkflowCollector {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            workflows_by_language: HashMap::new(),
        }
    }

    pub fn serialize_configs(&self, language: SupportedLanguages, output_path: PathBuf) {
        if let Some(workflows) = self.items().get(&language) {
            let configs: Vec<_> = workflows.iter().map(|w| w.config.clone()).collect();
            if let Ok(serialized_configs) = serde_json::to_string_pretty(&configs) {
                let _ = fs::write(output_path, serialized_configs);
            }
        }
    }
}

impl Collector for WorkflowCollector {
    type Item = Workflow;

    fn new() -> Self {
        Self::new()
    }

    fn collect(
        &mut self,
        path: PathBuf,
    ) -> Result<&HashMap<SupportedLanguages, Vec<Self::Item>>, std::io::Error> {
        let entries = fs::read_dir(path)?;

        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                // Each directory is potentially a workflow
                if let Ok(workflow) = Workflow::from_dir(entry.path()) {
                    self.workflows_by_language
                        .entry(workflow.language)
                        .or_default()
                        .push(workflow);
                }
            }
        }

        Ok(&self.workflows_by_language)
    }

    fn items(&self) -> &HashMap<SupportedLanguages, Vec<Self::Item>> {
        &self.workflows_by_language
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::languages::SupportedLanguages;
    use crate::framework::scripts::WorkflowConfig;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn it_works() {
        assert!(true);
    }

    #[test]
    fn test_script_from_file() {
        let script = Script::from_file(PathBuf::from("1.extract.py")).unwrap();
        assert_eq!(script.order, Some(1));
        assert_eq!(script.name, "extract");
        assert_eq!(script.language, SupportedLanguages::Python);
    }

    #[test]
    fn test_invalid_script_name() {
        assert!(Script::from_file(PathBuf::from("invalid")).is_err());
    }

    #[test]
    fn test_script_collector_collect() -> Result<(), std::io::Error> {
        let temp_dir = tempdir()?;
        let script_path = temp_dir.path().join("1.hello.py");

        // Create a test script file
        let mut file = File::create(&script_path)?;
        writeln!(file, "print('Hello from Python')")?;

        // Collect
        let mut collector = ScriptCollector::new();
        collector.collect(temp_dir.path().to_path_buf())?;

        // Verify
        let items = collector.items();
        assert!(items.contains_key(&SupportedLanguages::Python));
        let python_scripts = items.get(&SupportedLanguages::Python).unwrap();
        assert_eq!(python_scripts.len(), 1);
        Ok(())
    }

    #[test]
    fn test_workflow_collector() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;

        // Create a workflow directory structure
        let workflow_dir = temp_dir.path().join("test-workflow");
        fs::create_dir(&workflow_dir)?;

        let workflow_config_path = workflow_dir.join("config.toml");
        let workflow_config = WorkflowConfig::new("test-workflow".to_string());
        workflow_config.save(workflow_config_path)?;

        // Create some script files
        fs::write(workflow_dir.join("1.extract.py"), "print('Extract step')")?;
        fs::write(
            workflow_dir.join("2.transform.py"),
            "print('Transform step')",
        )?;

        // Create a parallel steps directory
        let parallel_dir = workflow_dir.join("3.parallel");
        fs::create_dir(&parallel_dir)?;
        fs::write(
            parallel_dir.join("process-a.py"),
            "print('Parallel step A')",
        )?;
        fs::write(
            parallel_dir.join("process-b.py"),
            "print('Parallel step B')",
        )?;

        // Collect workflows
        let mut collector = WorkflowCollector::new();
        collector.collect(temp_dir.path().to_path_buf())?;

        // Verify
        let items = collector.items();
        assert!(items.contains_key(&SupportedLanguages::Python));

        let python_workflows = items.get(&SupportedLanguages::Python).unwrap();
        assert_eq!(python_workflows.len(), 1);

        let workflow = &python_workflows[0];
        assert_eq!(workflow.name, "test-workflow");
        assert_eq!(workflow.scripts.len(), 4); // 2 main + 2 parallel scripts
        assert_eq!(workflow.config.name, workflow.name);

        Ok(())
    }
}
