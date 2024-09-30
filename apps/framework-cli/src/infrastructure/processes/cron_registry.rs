use anyhow::{anyhow, Result};
use log::{error, info};
use reqwest;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use toml;

#[derive(Error, Debug)]
pub enum CronError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Deserialize, Debug)]
pub struct CronJob {
    job_id: String,
    cron_spec: String,
    script_path: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct CronConfig {
    cron_jobs: Vec<CronJob>,
}

pub struct CronRegistry {
    scheduler: Arc<Mutex<JobScheduler>>,
}

impl CronRegistry {
    pub async fn new() -> Result<Self> {
        let scheduler = JobScheduler::new().await.map_err(|e| anyhow!(e))?;

        Ok(CronRegistry {
            scheduler: Arc::new(Mutex::new(scheduler)),
        })
    }

    pub async fn add_job<F>(&self, cron_expression: &str, task: F) -> Result<()>
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        let task = Arc::new(task);

        let job = Job::new_async(cron_expression, move |_uuid, _l| {
            let task = task.clone();
            Box::pin(async move {
                let _ = task();
            })
        })
        .map_err(|e| anyhow!(e))?;

        let scheduler = self.scheduler.lock().await;
        scheduler.add(job).await.map_err(|e| anyhow!(e))?;

        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        let scheduler = self.scheduler.lock().await;
        scheduler.start().await.map_err(|e| anyhow!(e))?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut scheduler = self.scheduler.lock().await;
        scheduler.shutdown().await.map_err(|e| anyhow!(e))?;

        Ok(())
    }

    pub async fn load_from_file(&self, file_path: &str) -> Result<()> {
        let toml_str = fs::read_to_string(file_path).map_err(|e| anyhow!(e))?;
        let config: CronConfig = toml::from_str(&toml_str).map_err(|e| anyhow!(e))?;
        info!("<cron> Loaded cron jobs from file: {}", file_path);
        info!("<cron> Cron jobs: {:?}", config.cron_jobs);

        for job in config.cron_jobs {
            let job_id = job.job_id;
            let cron_spec = job.cron_spec;
            let script_path = job.script_path;
            let url = job.url;

            // Ensure at least one of script_path or url is present
            if script_path.is_none() && url.is_none() {
                return Err(anyhow!(
                    "At least one of script_path or url must be present for job: {}",
                    job_id
                ));
            }

            self.add_job(&cron_spec, move || {
                if let Some(ref path) = script_path {
                    // Execute the script based on file extension
                    let extension = Path::new(path)
                        .extension()
                        .and_then(std::ffi::OsStr::to_str)
                        .unwrap_or("");

                    let output = match extension {
                        "js" => Command::new("node").arg(path).output(),
                        "ts" => Command::new("ts-node").arg(path).output(),
                        "py" => Command::new("python3").arg(path).output(),
                        _ => Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Unsupported file type",
                        )),
                    };

                    match output {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            if !stdout.is_empty() {
                                info!("<cron> Script stdout:\n{}", stdout);
                            }
                            if !stderr.is_empty() {
                                error!("<cron> Script stderr\n{}", stderr);
                            }
                            if !output.status.success() {
                                error!("<cron> Script exited with status:\n{}", output.status);
                            }
                        }
                        Err(e) => error!("<cron> Failed to execute script: {}", e),
                    }
                } else if let Some(ref url) = url.clone() {
                    info!("<cron> Calling URL: {}", url);
                    let url = url.to_string();
                    tokio::spawn(async move {
                        match reqwest::get(&url).await {
                            Ok(response) => {
                                info!("<cron> URL response status: {}", response.status())
                            }
                            Err(e) => error!("<cron> Failed to call URL: {}", e),
                        }
                    });
                }
                Ok(())
            })
            .await?;
        }

        Ok(())
    }

    pub async fn load_jobs(&self) -> Result<()> {
        let current_dir = env::current_dir().map_err(|e| anyhow!(e))?;
        let moose_cron_path = current_dir.join("moose.config.toml");

        if moose_cron_path.exists() {
            self.load_from_file(moose_cron_path.to_str().unwrap())
                .await?;
        } else {
            info!("<cron> moose.config.toml file not found in the project root directory. No jobs loaded.");
        }
        Ok(())
    }
}
