use crate::project::LanguageProjectConfig;
use async_trait::async_trait;

use crate::framework::python::checker::PythonChecker;
use crate::framework::typescript::checker::TypeScriptChecker;

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

pub async fn check_system_reqs(config: &LanguageProjectConfig) -> Result<(), CheckerError> {
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
