use anyhow::{anyhow, Result};
use log::info;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CronError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    // Add any other errors that your function might return.
}

// Ensure `CronError` implements `Send` and `Sync`
unsafe impl Send for CronError {}
unsafe impl Sync for CronError {}

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
        let json_str = fs::read_to_string(file_path).map_err(|e| anyhow!(e))?;
        let json: Value = serde_json::from_str(&json_str).map_err(|e| anyhow!(e))?;
        info!("<cron> Loaded cron jobs from file: {}", file_path);
        info!("<cron> Cron jobs: {}", json);
        let cron_jobs = json.get("cron_jobs").and_then(|j| j.as_array()).cloned();
        if let Some(cron_jobs) = cron_jobs {
            for job in cron_jobs {
                let job_id = job
                    .get("job_id")
                    .and_then(|j| j.as_str())
                    .map(String::from)
                    .ok_or_else(|| anyhow!("Missing job_id"))?;
                let cron_spec = job
                    .get("cron_spec")
                    .and_then(|j| j.as_str())
                    .map(String::from)
                    .ok_or_else(|| anyhow!("Missing cron_spec"))?;
                let script_path = job
                    .get("script_path")
                    .and_then(|j| j.as_str())
                    .map(String::from);
                let url = job.get("url").and_then(|j| j.as_str()).map(String::from);

                // Ensure at least one of script_path or url is present
                if script_path.is_none() && url.is_none() {
                    return Err(anyhow!(
                        "At least one of script_path or url must be present"
                    ));
                }

                self.add_job(&cron_spec, move || {
                    info!("<cron> Executing job: {}", job_id);
                    if let Some(ref path) = script_path {
                        info!("<cron> Script path: {}", path);
                        // Execute the script here
                    } else if let Some(ref url) = url {
                        info!("<cron> Calling URL: {}", url);
                        // Call the URL here
                    }
                    Ok(())
                })
                .await?;
            }
        }

        Ok(())
    }

    pub async fn load_jobs(&self) -> Result<()> {
        let moose_cron_path = Path::new("moose.json");
        if moose_cron_path.exists() {
            self.load_from_file(moose_cron_path.to_str().unwrap())
                .await?;
        } else {
            info!("<cron> moose.json file not found. No jobs loaded.");
        }
        Ok(())
    }
}
