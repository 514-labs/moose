use crate::cli::display::{show_message_wrapper, Message, MessageType};
use crate::project::{Project, ProjectFileError};

use log::error;
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
    let temporal_url = project.temporal_config.temporal_url();
    let scripts_dir = project.scripts_dir();

    let args: Vec<&str> = vec![
        scripts_dir.to_str().unwrap(),
        "--temporal-url",
        temporal_url.as_str(),
        "--client-cert",
        project.temporal_config.client_cert.as_str(),
        "--client-key",
        project.temporal_config.client_key.as_str(),
        "--api-key",
        project.temporal_config.api_key.as_str(),
    ];

    let mut scripts_process = bin::run(SCRIPTS_BIN, &project_path, &args)?;

    let stdout = scripts_process
        .stdout
        .take()
        .expect("Scripts process did not have a handle to stdout");

    let stderr = scripts_process
        .stderr
        .take()
        .expect("Scripts process did not have a handle to stderr");

    // Use buffered stdout reader macro to preserve service context
    crate::buffered_stdout_reader!(stdout, 100, "Workflow");

    let mut stderr_reader = BufReader::new(stderr).lines();

    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            error!("{}", line);
            show_message_wrapper(
                MessageType::Error,
                Message {
                    action: "Workflow".to_string(),
                    details: line.to_string(),
                },
            );
        }
    });

    Ok(scripts_process)
}
