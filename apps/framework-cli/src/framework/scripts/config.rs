use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowConfig {
    // Basic workflow configuration
    pub name: String,
    #[serde(default = "default_schedule")]
    pub schedule: String,
    #[serde(default = "default_retries")]
    pub retries: u32,
    #[serde(default = "default_timeout")]
    pub timeout: String,

    // Optional steps configuration
    #[serde(default)]
    pub steps: Option<Vec<String>>,
}

impl WorkflowConfig {
    pub fn new(name: String) -> Self {
        WorkflowConfig {
            name,
            schedule: default_schedule(),
            retries: default_retries(),
            timeout: default_timeout(),
            steps: None,
        }
    }

    pub fn with_steps(name: String, steps: Vec<String>) -> Self {
        let mut config = Self::new(name);
        config.steps = Some(steps);
        config
    }

    pub fn save(&self, path: PathBuf) -> std::io::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, content)
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
        let config = WorkflowConfig::new("daily-etl".to_string());
        assert_eq!(config.name, "daily-etl");
        assert_eq!(config.schedule, "0 0 * * *");
        assert_eq!(config.retries, 3);

        // Test TOML serialization directly
        let toml_str = toml::to_string_pretty(&config).unwrap();
        println!("Direct TOML serialization:\n{}", toml_str);
        assert!(toml_str.contains("name = 'daily-etl'"));
    }
}
