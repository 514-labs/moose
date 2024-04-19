use std::path::PathBuf;
use std::process::{Command, Stdio};

use log::error;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, from_str};

use crate::project::Project;
use crate::utilities::constants::{CLI_VERSION, REDPANDA_CONTAINER_NAME};

static COMPOSE_FILE: &str = include_str!("docker-compose.yml");

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

pub fn list_containers() -> std::io::Result<Vec<ContainerRow>> {
    let child = Command::new("docker")
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
        Err(anyhow::anyhow!("Failed to strop containers"))
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

pub fn start_containers(project: &Project) -> anyhow::Result<()> {
    let console_version = if cfg!(debug_assertions) {
        "latest"
    } else {
        CLI_VERSION
    };

    let console_pull_policy = if console_version == "latest" {
        "always"
    } else {
        "missing"
    };

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
            "CONSOLE_HOST_PORT",
            project.console_config.host_port.to_string(),
        )
        .env(
            "CLICKHOUSE_HOST_PORT",
            project.clickhouse_config.host_port.to_string(),
        )
        .env(
            "CLICKHOUSE_POSTGRES_PORT",
            project.clickhouse_config.postgres_port.to_string(),
        )
        .env("CLICKHOUSE_VERSION", "24.1.3") // https://github.com/ClickHouse/ClickHouse/issues/60020
        .env("CONSOLE_VERSION", console_version)
        .env("CONSOLE_PULL_POLICY", console_pull_policy)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        error!(
            "Failed to start containers: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Err(anyhow::anyhow!("Failed to start containers"))
    } else {
        Ok(())
    }
}

pub fn create_compose_file(project: &Project) -> std::io::Result<()> {
    let compose_file = project.internal_dir()?.join("docker-compose.yml");
    std::fs::write(compose_file, COMPOSE_FILE)
}

pub fn run_rpk_cluster_info(project_name: &str) -> anyhow::Result<()> {
    let child = Command::new("docker")
        .arg("exec")
        .arg(format!("{}-{}", project_name, REDPANDA_CONTAINER_NAME))
        .arg("rpk")
        .arg("cluster")
        .arg("info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if !output.status.success() {
        error!(
            "Failed to stop containers: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Err(anyhow::anyhow!("Failed to strop containers"))
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
            "Failed to run dockerx build",
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
