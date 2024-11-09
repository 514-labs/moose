use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, from_str};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::project::Project;
use crate::utilities::constants::REDPANDA_CONTAINER_NAME;

static COMPOSE_FILE: &str = include_str!("docker-compose.yml");

#[derive(Debug, thiserror::Error)]
#[error("Failed to create or delete project files")]
#[non_exhaustive]
pub enum DockerError {
    ProjectFile(#[from] crate::project::ProjectFileError),
    IO(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerRow {
    pub command: String,
    pub created_at: String,
    #[serde(rename = "ID")]
    pub id: String,
    pub image: String,
    pub labels: String,
    pub local_volumes: String,
    pub mounts: String,
    pub names: String,
    pub networks: String,
    pub ports: String,
    pub running_for: String,
    pub size: String,
    pub state: String,
    pub status: String,
    pub health: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkRow {
    pub created_at: String,
    pub driver: String,
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "IPv6")]
    pub ipv6: String,
    pub internal: String,
    pub labels: String,
    pub name: String,
    pub scope: String,
}

pub fn list_containers(project: &Project) -> std::io::Result<Vec<ContainerRow>> {
    let child = compose_command(project)
        .arg("ps")
        .arg("-a")
        .arg("--no-trunc")
        .arg("--format")
        .arg("json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to list Docker containers",
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let containers: Vec<ContainerRow> = output_str
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| from_str(line).expect("Failed to parse container row"))
        .collect();

    Ok(containers)
}

pub fn list_container_names() -> std::io::Result<Vec<String>> {
    let child = Command::new("docker")
        .arg("ps")
        .arg("--format")
        .arg("{{json .Names}}")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to list Docker container names",
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let containers: Vec<String> = output_str
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| from_str(line).expect("Failed to parse container row"))
        .collect();

    Ok(containers)
}

pub fn stop_containers(project: &Project) -> anyhow::Result<()> {
    let child = compose_command(project)
        .arg("down")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        error!(
            "Failed to stop containers: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Err(anyhow::anyhow!("Failed to stop containers"))
    } else {
        Ok(())
    }
}

fn compose_command(project: &Project) -> Command {
    let mut command = Command::new("docker");

    command
        .arg("compose")
        .arg("-f")
        .arg(project.internal_dir().unwrap().join("docker-compose.yml"))
        .arg("-p")
        .arg(project.name().to_lowercase());
    command
}

lazy_static! {
    pub static ref PORT_ALLOCATED_REGEX: Regex =
        Regex::new("Bind for \\d+.\\d+.\\d+.\\d+:(\\d+) failed: port is already allocated")
            .unwrap();
}

pub fn start_containers(project: &Project) -> anyhow::Result<()> {
    project.create_internal_redpanda_volume()?;
    project.create_internal_clickhouse_volume()?;

    let child = compose_command(project)
        .arg("up")
        .arg("-d")
        .env("DB_NAME", project.clickhouse_config.db_name.clone())
        .env("CLICKHOUSE_USER", project.clickhouse_config.user.clone())
        .env(
            "CLICKHOUSE_PASSWORD",
            project.clickhouse_config.password.clone(),
        )
        .env(
            "CLICKHOUSE_HOST_PORT",
            project.clickhouse_config.host_port.to_string(),
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        error!("Failed to start containers: {}", error_message);

        let mapped_error_message =
            if let Some(stuff) = PORT_ALLOCATED_REGEX.captures(&error_message) {
                format!("Port {} already in use.", stuff.get(1).unwrap().as_str())
            } else {
                error_message.to_string()
            };

        Err(anyhow::anyhow!(
            "Failed to start containers: {}",
            mapped_error_message
        ))
    } else {
        Ok(())
    }
}

pub fn tail_container_logs(project: &Project, container_name: &str) -> anyhow::Result<()> {
    let full_container_name = format!("{}-{}", project.name().to_lowercase(), container_name);
    let container_id = get_container_id(&full_container_name)?;

    let mut child = tokio::process::Command::new("docker")
        .arg("logs")
        .arg("--follow")
        .arg(container_id)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().ok_or(anyhow::anyhow!(
        "Failed to get stdout for {}",
        full_container_name
    ))?;

    let stderr = child.stderr.take().ok_or(anyhow::anyhow!(
        "Failed to get stderr for {}",
        full_container_name
    ))?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let log_identifier_stdout = full_container_name.clone();
    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            info!("<{}> {}", log_identifier_stdout, line);
        }
    });

    let log_identifier_stderr = full_container_name.clone();
    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            error!("<{}> {}", log_identifier_stderr, line);
        }
    });

    Ok(())
}

fn get_container_id(container_name: &str) -> anyhow::Result<String> {
    let child = Command::new("docker")
        .arg("ps")
        .arg("-aqf")
        .arg(format!("name={}", container_name))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        error!(
            "Failed to get container id: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Err(anyhow::anyhow!(format!(
            "Failed to get container id for {}",
            container_name
        )))
    } else {
        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(container_id)
    }
}

pub fn create_compose_file(project: &Project) -> Result<(), DockerError> {
    let compose_file = project.internal_dir()?.join("docker-compose.yml");
    Ok(std::fs::write(compose_file, COMPOSE_FILE)?)
}

pub fn run_rpk_cluster_info(project_name: &str, attempts: usize) -> anyhow::Result<()> {
    let child = Command::new("docker")
        .arg("exec")
        .arg(format!(
            "{}-{}",
            project_name.to_lowercase(),
            REDPANDA_CONTAINER_NAME
        ))
        .arg("rpk")
        .arg("cluster")
        .arg("info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        if attempts > 0 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            Ok(run_rpk_cluster_info(project_name, attempts - 1)?)
        } else {
            error!(
                "Failed to run redpanda cluster info: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            Err(anyhow::anyhow!("Failed to run redpanda cluster info"))
        }
    } else {
        Ok(())
    }
}

pub fn run_rpk_command(project_name: &str, args: Vec<String>) -> std::io::Result<String> {
    let child = Command::new("docker")
        .arg("exec")
        .arg(format!("{}-{}", project_name, REDPANDA_CONTAINER_NAME))
        .arg("rpk")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else if output.stderr.is_empty() {
        if output.stdout.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No output from command",
            ));
        }

        if String::from_utf8_lossy(&output.stdout).contains("TOPIC_ALREADY_EXISTS") {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }

        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stdout),
        ));
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "stdout: {}, stderr: {}",
                String::from_utf8_lossy(&output.stdout),
                &String::from_utf8_lossy(&output.stderr)
            ),
        ))
    }
}

pub fn check_status() -> std::io::Result<Vec<String>> {
    let child = Command::new("docker")
        .arg("info")
        .arg("--format")
        .arg("{{json .ServerErrors}}")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get Docker info",
        ));
    }

    let errors: Option<Vec<String>> = from_slice(&output.stdout)?;
    Ok(errors.unwrap_or_default())
}

pub fn buildx(
    directory: &PathBuf,
    version: &str,
    architecture: &str,
    binarylabel: &str,
) -> std::io::Result<Vec<String>> {
    let child = Command::new("docker")
        .current_dir(directory)
        .arg("buildx")
        .arg("build")
        .arg("--build-arg")
        .arg(format!(
            "DOWNLOAD_URL=https://github.com/514-labs/moose/releases/download/v{}/moose-cli-{}",
            version, binarylabel
        ))
        .arg("--platform")
        .arg(architecture)
        .arg("--load")
        .arg("--no-cache")
        .arg("-t")
        // Using latest for the version tag as the user might want to use its own
        // version tag, this makes it easy for the automation to re-label the image.
        .arg(format!("moose-df-deployment-{}:latest", binarylabel))
        .arg(".")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let containers: Vec<String> = output_str
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| from_str(line).expect("Failed to parse container row"))
        .collect();
    Ok(containers)
}
