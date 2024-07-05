use std::path::PathBuf;

use log::info;
use tokio::process::Child;

use crate::{
    framework::{languages::SupportedLanguages, python, typescript},
    infrastructure::olap::clickhouse::config::ClickHouseConfig,
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
}

impl ConsumptionProcessRegistry {
    pub fn new(
        language: SupportedLanguages,
        clickhouse_config: ClickHouseConfig,
        dir: PathBuf,
    ) -> Self {
        Self {
            api_process: Option::None,
            language,
            dir,
            clickhouse_config,
        }
    }

    pub fn start(&mut self) -> Result<(), ConsumptionError> {
        info!("Starting consumption API...");

        let child = match self.language {
            SupportedLanguages::Python => {
                python::consumption::run(self.clickhouse_config.clone(), &self.dir)
            }
            SupportedLanguages::Typescript => {
                typescript::consumption::run(self.clickhouse_config.clone(), &self.dir)
            }
        }?;

        self.api_process = Some(child);

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), ConsumptionError> {
        info!("Stopping consumption...");

        match &self.api_process {
            Some(child) => kill_child(child).await?,
            None => (),
        };

        self.api_process = None;

        Ok(())
    }
}
