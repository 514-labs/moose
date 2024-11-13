use crate::project::Project;
use crate::utilities::constants::TSCONFIG_JSON;
use anyhow::{anyhow, Result};
use log::{error, info};
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(Error, Debug)]
pub enum CronError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CronJob {
    job_id: String,
    cron_spec: String,
    script_path: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CronMetric {
    pub job_id: String,
    pub timestamp: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

pub struct CronRegistry {
    scheduler: Arc<Mutex<JobScheduler>>,
    registered_jobs: Arc<Mutex<HashSet<String>>>,
    jobs_registered: Arc<Mutex<bool>>,
    metrics: Arc<Mutex<Vec<CronMetric>>>,
}

impl CronRegistry {
    pub async fn new() -> Result<Self> {
        let scheduler = JobScheduler::new().await.map_err(|e| anyhow!(e))?;

        Ok(CronRegistry {
            scheduler: Arc::new(Mutex::new(scheduler)),
            registered_jobs: Arc::new(Mutex::new(HashSet::new())),
            jobs_registered: Arc::new(Mutex::new(false)),
            metrics: Arc::new(Mutex::new(Vec::new())),
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

    pub async fn register_jobs(&self, project: &Project) -> Result<()> {
        let mut jobs_registered = self.jobs_registered.lock().await;
        if *jobs_registered {
            info!("<cron> Jobs have already been registered, skipping");
            return Ok(());
        }

        info!("<cron> Attempting to register cron jobs from project configuration");
        info!("<cron> Cron jobs: {:?}", project.cron_jobs);
        let project_path = project.project_location.clone();

        for job in &project.cron_jobs {
            let job_id = job.job_id.clone();

            // Check if the job is already registered
            let mut registered_jobs = self.registered_jobs.lock().await;
            if registered_jobs.contains(&job_id) {
                info!("<cron> Job {} is already registered, skipping", job_id);
                continue;
            }

            let cron_spec = job.cron_spec.clone();
            let script_path = job.script_path.clone();
            let url = job.url.clone();

            // Ensure at least one of script_path or url is present
            if script_path.is_none() && url.is_none() {
                return Err(anyhow!(
                    "At least one of script_path or url must be present for job: {}",
                    job_id
                ));
            }

            let project_path_clone = project_path.clone();
            let metrics = self.metrics.clone();
            let job_id_for_metric = job_id.clone();

            self.add_job(&cron_spec, move || {
                let metrics = metrics.clone();
                let mut success = true;
                let mut error_msg = None;

                if let Some(ref path) = script_path {
                    // Execute the script based on file extension
                    let extension = Path::new(path)
                        .extension()
                        .and_then(std::ffi::OsStr::to_str)
                        .unwrap_or("");

                    let output = match extension {
                        "js" => Command::new("node").arg(path).output(),
                        "ts" => {
                            let path =
                                env::var("PATH").unwrap_or_else(|_| "/usr/local/bin".to_string());
                            let bin_path = format!(
                                "{}:{}/node_modules/.bin",
                                path,
                                project_path_clone.to_str().unwrap()
                            );
                            let ts_script_path =
                                project_path_clone.join(script_path.as_ref().unwrap());
                            Command::new("moose-exec")
                                .arg(ts_script_path.to_str().unwrap())
                                .env("TS_NODE_PROJECT", project_path_clone.join(TSCONFIG_JSON))
                                .env("PATH", bin_path)
                                .output()
                        }
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
                                success = false;
                                error_msg =
                                    Some(format!("Script exited with status: {}", output.status));
                                error!("<cron> Script exited with status:\n {}", output.status);
                            }
                        }
                        Err(e) => {
                            success = false;
                            error_msg = Some(e.to_string());
                            error!("<cron> Failed to execute script: {}", e);
                        }
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

                // Record metrics
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let metric = CronMetric {
                    job_id: job_id_for_metric.clone(),
                    timestamp,
                    success,
                    error_message: error_msg,
                };

                tokio::spawn(async move {
                    let mut metrics = metrics.lock().await;
                    metrics.push(metric);
                });

                Ok(())
            })
            .await?;

            // Add the job_id to the set of registered jobs
            registered_jobs.insert(job_id);
        }

        *jobs_registered = true;
        Ok(())
    }

    pub async fn get_metrics(&self) -> Vec<CronMetric> {
        self.metrics.lock().await.clone()
    }
}
