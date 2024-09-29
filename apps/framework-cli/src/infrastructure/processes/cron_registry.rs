use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

pub struct CronRegistry {
    scheduler: Arc<Mutex<JobScheduler>>,
}

impl CronRegistry {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let scheduler = JobScheduler::new().await?;

        Ok(CronRegistry {
            scheduler: Arc::new(Mutex::new(scheduler)),
        })
    }

    pub async fn add_job<F>(&self, cron_expression: &str, task: F) -> Result<(), JobSchedulerError>
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        let task = Arc::new(task);

        let job = Job::new_async(cron_expression, move |_uuid, _l| {
            let task = task.clone();
            Box::pin(async move {
                let _ = task();
            })
        })?;

        let scheduler = self.scheduler.lock().await;
        scheduler.add(job).await?;

        Ok(())
    }

    pub async fn start(&self) -> Result<(), anyhow::Error> {
        let scheduler = self.scheduler.lock().await;
        scheduler.start().await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), JobSchedulerError> {
        let mut scheduler = self.scheduler.lock().await;
        scheduler.shutdown().await?;

        Ok(())
    }

    pub async fn load_from_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json_str = fs::read_to_string(file_path)?;
        let json: Value = serde_json::from_str(&json_str)?;

        let cron_jobs = json.get("cron_jobs").and_then(|j| j.as_array()).cloned();
        if let Some(cron_jobs) = cron_jobs {
            for job in cron_jobs {
                let job_id = job
                    .get("job_id")
                    .and_then(|j| j.as_str())
                    .map(String::from)
                    .ok_or("Missing job_id")?;
                let cron_spec = job
                    .get("cron_spec")
                    .and_then(|j| j.as_str())
                    .map(String::from)
                    .ok_or("Missing cron_spec")?;
                let script_path = job
                    .get("script_path")
                    .and_then(|j| j.as_str())
                    .map(String::from);
                let url = job.get("url").and_then(|j| j.as_str()).map(String::from);

                // Ensure at least one of script_path or url is present
                if script_path.is_none() && url.is_none() {
                    return Err("At least one of script_path or url must be present".into());
                }

                self.add_job(&cron_spec, move || {
                    println!("Executing job: {}", job_id);
                    if let Some(ref path) = script_path {
                        // Borrowing script_path
                        println!("Script path: {}", path);
                        // Execute the script here
                    } else if let Some(ref url) = url {
                        // Borrow url
                        println!("Calling URL: {}", url);
                        // Call the URL here
                    }
                    Ok(())
                })
                .await?;
            }
        }

        Ok(())
    }

    pub async fn load_jobs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let moose_cron_path = Path::new("moose.json");
        if moose_cron_path.exists() {
            self.load_from_file(moose_cron_path.to_str().unwrap())
                .await?;
        } else {
            println!("moose.json file not found. No jobs loaded.");
        }
        Ok(())
    }
}
