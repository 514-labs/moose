use anyhow::Result;
use log::{error, info, warn};
use std::fs;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use crate::cli::display::{Message, MessageType};
use crate::project::{Project, ProjectFileError};
use crate::show_message;
use crate::utilities::constants::PYTHON_WORKER_WRAPPER_PACKAGE_NAME;

use super::executor::{run_python_program, PythonProgram};

#[derive(Debug, thiserror::Error)]
pub enum WorkerProcessError {
    #[error("Failed to start worker process")]
    IOError(#[from] std::io::Error),

    #[error("Failed to create library files")]
    ProjectFileError(#[from] ProjectFileError),
}

pub async fn start_worker(project: &Project) -> Result<Child, WorkerProcessError> {
    let scripts_dir = project.scripts_dir();

    // Create the wrapper lib files inside the .moose directory
    let internal_dir = project.internal_dir()?;
    let python_worker_lib_dir = internal_dir.join(PYTHON_WORKER_WRAPPER_PACKAGE_NAME);

    // Create the directory if it doesn't exist
    if !python_worker_lib_dir.exists() {
        fs::create_dir(&python_worker_lib_dir)?;
    }

    // Overwrite the wrapper files
    fs::write(
        python_worker_lib_dir.join("__init__.py"),
        include_str!("./wrappers/scripts/python_worker_wrapper/__init__.py"),
    )?;
    fs::write(
        python_worker_lib_dir.join("activity.py"),
        include_str!("./wrappers/scripts/python_worker_wrapper/activity.py"),
    )?;
    fs::write(
        python_worker_lib_dir.join("worker.py"),
        include_str!("./wrappers/scripts/python_worker_wrapper/worker.py"),
    )?;
    fs::write(
        python_worker_lib_dir.join("workflow.py"),
        include_str!("./wrappers/scripts/python_worker_wrapper/workflow.py"),
    )?;
    fs::write(
        python_worker_lib_dir.join("logging.py"),
        include_str!("./wrappers/scripts/python_worker_wrapper/logging.py"),
    )?;
    fs::write(
        python_worker_lib_dir.join("types.py"),
        include_str!("./wrappers/scripts/python_worker_wrapper/types.py"),
    )?;
    fs::write(
        python_worker_lib_dir.join("serialization.py"),
        include_str!("./wrappers/scripts/python_worker_wrapper/serialization.py"),
    )?;

    let mut worker_process = run_python_program(PythonProgram::OrchestrationWorker {
        args: vec![scripts_dir.to_string_lossy().to_string()],
    })?;

    let stdout = worker_process
        .stdout
        .take()
        .expect("Worker process did not have a handle to stdout");

    let stderr = worker_process
        .stderr
        .take()
        .expect("Worker process did not have a handle to stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                let level = parts[0].trim();
                let message = parts[2].trim();
                match level {
                    "INFO" => info!("{}", message),
                    "WARNING" => warn!("{}", message),
                    "ERROR" => {
                        error!("{}", message);
                        show_message!(
                            MessageType::Error,
                            Message {
                                action: "WorkflowActivityError".to_string(),
                                details: message.to_string(),
                            },
                            true
                        );
                    }
                    _ => info!("{}", message),
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            error!("{}", line);
            show_message!(
                MessageType::Error,
                Message {
                    action: "WorkflowActivityError".to_string(),
                    details: line.to_string(),
                },
                true
            );
        }
    });

    Ok(worker_process)
}
