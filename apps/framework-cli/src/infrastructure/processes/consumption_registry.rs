use std::path::PathBuf;

use log::info;
use tokio::process::Child;

use crate::{
    framework::{languages::SupportedLanguages, python, typescript},
    infrastructure::olap::clickhouse::config::ClickHouseConfig,
    project::{JwtConfig, Project, ProjectFileError},
    utilities::system::{kill_child, KillProcessError},
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConsumptionError {
    #[error("Failed to start/stop the consumption process")]
    IoError(#[from] std::io::Error),

    #[error("Kill process Error")]
    KillProcessError(#[from] KillProcessError),

    #[error("Failed to create library files")]
    ProjectFileError(#[from] ProjectFileError),
}

pub struct ConsumptionProcessRegistry {
    api_process: Option<Child>,
    clickhouse_config: ClickHouseConfig,
    dir: PathBuf,
    language: SupportedLanguages,
    project_path: PathBuf,
    jwt_config: Option<JwtConfig>,
    project: Project,
}

impl ConsumptionProcessRegistry {
    pub fn new(
        language: SupportedLanguages,
        clickhouse_config: ClickHouseConfig,
        jwt_config: Option<JwtConfig>,
        dir: PathBuf,
        project_path: PathBuf,
        project: Project,
    ) -> Self {
        Self {
            api_process: Option::None,
            language,
            dir,
            clickhouse_config,
            project_path,
            jwt_config,
            project,
        }
    }

    pub fn start(&mut self) -> Result<(), ConsumptionError> {
        info!("Starting consumption API...");

        let child = match self.language {
            SupportedLanguages::Python => python::consumption::run(
                self.project.clone(),
                self.clickhouse_config.clone(),
                self.jwt_config.clone(),
                &self.dir,
            ),
            SupportedLanguages::Typescript => typescript::consumption::run(
                self.project.clone(),
                self.clickhouse_config.clone(),
                self.jwt_config.clone(),
                &self.dir,
                &self.project_path,
            ),
        }?;

        self.api_process = Some(child);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), ConsumptionError> {
        info!("Stopping consumption...");

        if let Some(child) = &self.api_process {
            kill_child(child).await?
        };

        self.api_process = None;

        Ok(())
    }
}
