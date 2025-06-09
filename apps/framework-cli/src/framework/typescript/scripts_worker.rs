use crate::cli::display::{show_message_wrapper, Message, MessageType};
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::project::{Project, ProjectFileError};

use log::{debug, error, info, warn};
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

pub async fn start_worker(
    project: &Project,
    clickhouse_config: &ClickHouseConfig,
) -> Result<Child, WorkerProcessError> {
    log::debug!("start_worker");

    let project_path = project.project_location.clone();
    let temporal_url = project.temporal_config.temporal_url();
    let scripts_dir = project.scripts_dir();
    let host_port = clickhouse_config.host_port.to_string();

    log::debug!("clickhouse_config: {:?}", clickhouse_config);

    let mut string_args = vec![
        scripts_dir.to_str().unwrap().to_string(),
        clickhouse_config.db_name.clone(),
        clickhouse_config.host.clone(),
        host_port,
        clickhouse_config.user.clone(),
        clickhouse_config.password.clone(),
    ];

    if clickhouse_config.use_ssl {
        string_args.push("--clickhouse-use-ssl".to_string());
    }

    string_args.extend([
        "--temporal-url".to_string(),
        temporal_url,
        "--client-cert".to_string(),
        project.temporal_config.client_cert.clone(),
        "--client-key".to_string(),
        project.temporal_config.client_key.clone(),
        "--api-key".to_string(),
        project.temporal_config.api_key.clone(),
    ]);

    let args: Vec<&str> = string_args.iter().map(|s| s.as_str()).collect();

    log::debug!("args: {:?}", args);

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
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 2 {
                let level = parts[0].trim();
                let message = parts[1].trim();
                match level {
                    "INFO" => info!("{}", message),
                    "WARN" => warn!("{}", message),
                    "DEBUG" => debug!("{}", message),
                    "ERROR" => {
                        error!("{}", message);
                        show_message_wrapper(
                            MessageType::Error,
                            Message {
                                action: "Workflow".to_string(),
                                details: message.to_string(),
                            },
                        );
                    }
                    _ => info!("{}", message),
                }
            } else {
                info!("{}", line);
            }
        }
    });

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
