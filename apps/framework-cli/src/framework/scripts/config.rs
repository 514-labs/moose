use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowConfig {
    // Basic workflow configuration
    pub name: String,
    #[serde(default = "default_schedule")]
    pub schedule: String,
    #[serde(default = "default_retries")]
    pub retries: u32,
    #[serde(default = "default_timeout")]
    pub timeout: String,

    // Optional tasks configuration
    #[serde(default)]
    pub tasks: Option<Vec<String>>,
}

impl WorkflowConfig {
    pub fn new(name: String) -> Self {
        WorkflowConfig {
            name,
            schedule: default_schedule(),
            retries: default_retries(),
            timeout: default_timeout(),
            tasks: None,
        }
    }

    pub fn with_tasks(name: String, tasks: Vec<String>) -> Self {
        let mut config = Self::new(name);
        config.tasks = Some(tasks);
        config
    }

    pub fn save(&self, path: PathBuf) -> std::io::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, content)
    }

    pub fn from_file(path: PathBuf) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let config: WorkflowConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

// Default values functions
fn default_schedule() -> String {
    "".to_string() // Empty string means no schedule
}

fn default_retries() -> u32 {
    3
}

fn default_timeout() -> String {
    "1h".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = WorkflowConfig::new("daily-etl".to_string());
        config.save(config_path.clone()).unwrap();

        let content = std::fs::read_to_string(config_path).unwrap();
        println!("Generated TOML content:\n{}", content); // Debug output

        // Check the raw content first
        assert!(!content.is_empty(), "Config file should not be empty");

        // Parse the TOML content back to verify structure
        let parsed_config: WorkflowConfig = toml::from_str(&content).unwrap();
        assert_eq!(
            parsed_config.name, "daily-etl",
            "Name should match in parsed config"
        );

        // Now check the string content with single quotes
        assert!(
            content.contains("name = 'daily-etl'"),
            "Expected name = 'daily-etl' in content:\n{}",
            content
        );
    }

    #[test]
    fn test_basic_config_creation() {
        let config = WorkflowConfig {
            name: "test".to_string(),
            schedule: "0 0 * * *".to_string(),
            ..Default::default()
        };

        assert_eq!(config.name, "test");
        assert_eq!(config.schedule, "0 0 * * *");
    }
}
