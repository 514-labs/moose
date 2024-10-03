use async_trait::async_trait;
use tokio::process::Command;

use crate::project::LanguageProjectConfig;

#[derive(Debug, thiserror::Error)]
pub enum CheckerError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Not supported: {0}")]
    NotSupported(String),
}

#[async_trait]
pub trait SystemChecker {
    async fn validate(&self) -> Result<(), CheckerError>;
}

pub struct PythonChecker {
    pub required_version: String,
}

#[async_trait]
impl SystemChecker for PythonChecker {
    async fn validate(&self) -> Result<(), CheckerError> {
        check_python_version(&self.required_version).await
    }
}

pub struct TypeScriptChecker;

#[async_trait]
impl SystemChecker for TypeScriptChecker {
    async fn validate(&self) -> Result<(), CheckerError> {
        Ok(())
    }
}

async fn check_python_version(required_version: &str) -> Result<(), CheckerError> {
    let output = Command::new("python3").arg("--version").output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckerError::NotSupported(format!(
            "Failed to get Python version: {}",
            stderr
        )));
    }

    let version_str = std::str::from_utf8(&output.stdout).unwrap_or("");
    let version_parts: Vec<&str> = version_str.split_whitespace().collect();
    if version_parts.len() < 2 {
        return Err(CheckerError::NotSupported(format!(
            "Failed to get Python version. Found version: {}",
            version_str
        )));
    }

    let current_version = version_parts[1];
    if compare_versions(current_version, required_version) {
        Ok(())
    } else {
        Err(CheckerError::NotSupported(format!(
            "Python version {} is not supported. Required version is {}+",
            current_version, required_version
        )))
    }
}

fn compare_versions(current_version: &str, required_version: &str) -> bool {
    let current_parts: Vec<u32> = current_version
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();

    let required_parts: Vec<u32> = required_version
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();

    for (current, required) in current_parts.iter().zip(required_parts.iter()) {
        match current.cmp(required) {
            std::cmp::Ordering::Greater => return true,
            std::cmp::Ordering::Less => return false,
            std::cmp::Ordering::Equal => continue,
        }
    }

    current_parts.len() >= required_parts.len()
}

pub struct Checker;

impl Checker {
    pub async fn validate_system_reqs(config: &LanguageProjectConfig) -> Result<(), CheckerError> {
        match config {
            LanguageProjectConfig::Typescript(_) => {
                let checker = TypeScriptChecker;
                checker.validate().await
            }
            LanguageProjectConfig::Python(_) => {
                let checker = PythonChecker {
                    required_version: "3.12".to_string(),
                };
                checker.validate().await
            }
        }
    }
}
