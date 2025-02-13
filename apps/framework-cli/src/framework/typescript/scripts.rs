use crate::project::{Project, ProjectFileError};

use log::{error, info};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use super::bin;

#[derive(Debug, thiserror::Error)]
pub enum WorkerProcessError {
    #[error("Failed to start worker process")]
    IOError(#[from] std::io::Error),

    #[error("Failed to create library files")]
    ProjectFileError(#[from] ProjectFileError),
}

const SCRIPTS_BIN: &str = "scripts";

pub async fn start_worker(project: &Project) -> Result<Child, WorkerProcessError> {
    let project_path = project.project_location.clone();
    let scripts_dir = project.scripts_dir();

    let args: Vec<&str> = vec![scripts_dir.to_str().unwrap()];

    let mut scripts_process = bin::run(SCRIPTS_BIN, &project_path, &args)?;

    let stdout = scripts_process
        .stdout
        .take()
        .expect("Scripts process did not have a handle to stdout");

    let stderr = scripts_process
        .stderr
        .take()
        .expect("Scripts process did not have a handle to stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            info!("{}", line);
        }
    });

    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            error!("{}", line);
        }
    });

    Ok(scripts_process)
}
