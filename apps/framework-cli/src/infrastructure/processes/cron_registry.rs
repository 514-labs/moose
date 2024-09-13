use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};

pub struct CronRegistry {
    scheduler: Arc<Mutex<JobScheduler>>,
}

impl CronRegistry {
    pub async fn new() -> Result<Self, JobSchedulerError> {
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

    pub async fn start(&self) -> Result<(), JobSchedulerError> {
        let scheduler: Arc<Mutex<JobScheduler>> = self.scheduler.clone();
        task::spawn(async move {
            let scheduler = scheduler.lock().await;
            scheduler.start().await?;

            Ok::<(), JobSchedulerError>(())
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), JobSchedulerError> {
        let mut scheduler = self.scheduler.lock().await;
        scheduler.shutdown().await?;

        Ok(())
    }
}
