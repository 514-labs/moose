use std::path::PathBuf;

use log::info;
use tokio::process::Child;

use crate::{
    framework::{languages::SupportedLanguages, python, typescript},
    infrastructure::olap::clickhouse::config::ClickHouseConfig,
    project::JwtConfig,
    utilities::system::{kill_child, KillProcessError},
};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConsumptionError {
    #[error("Failed to start/stop the consumption process")]
    IoError(#[from] std::io::Error),

    #[error("Kill process Error")]
    KillProcessError(#[from] KillProcessError),
}

pub struct ConsumptionProcessRegistry {
    api_process: Option<Child>,
    clickhouse_config: ClickHouseConfig,
    dir: PathBuf,
    language: SupportedLanguages,
    project_path: PathBuf,
    jwt_config: Option<JwtConfig>,
    temporal_url: String,
}

impl ConsumptionProcessRegistry {
    pub fn new(
        language: SupportedLanguages,
        clickhouse_config: ClickHouseConfig,
        jwt_config: Option<JwtConfig>,
        dir: PathBuf,
        project_path: PathBuf,
        temporal_url: String,
    ) -> Self {
        Self {
            api_process: Option::None,
            language,
            dir,
            clickhouse_config,
            project_path,
            jwt_config,
            temporal_url,
        }
    }

    pub fn start(&mut self) -> Result<(), ConsumptionError> {
        info!("Starting consumption API...");

        let child = match self.language {
            SupportedLanguages::Python => python::consumption::run(
                self.clickhouse_config.clone(),
                self.jwt_config.clone(),
                &self.dir,
                &self.temporal_url,
            ),
            SupportedLanguages::Typescript => typescript::consumption::run(
                self.clickhouse_config.clone(),
                self.jwt_config.clone(),
                &self.dir,
                &self.project_path,
                &self.temporal_url,
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
