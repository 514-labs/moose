use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};
use serde_json::{from_slice, from_str};

use crate::project::Project;
use crate::utilities::constants::{CLI_VERSION, REDPANDA_CONTAINER_NAME};

static COMPOSE_FILE: &str = r#"
services:
  redpanda:
    image: docker.redpanda.com/redpandadata/redpanda:latest
    ports:
      - "9092:9092"
      - "19092:19092"
      - "9644:9644"
    volumes:
      - .panda_house:/tmp/panda_house
    command:
      - redpanda
      - start
      - --kafka-addr=internal://0.0.0.0:9092,external://0.0.0.0:19092
      - --advertise-kafka-addr=internal://redpanda:9092,external://localhost:19092
      - --pandaproxy-addr=internal://0.0.0.0:8082,external://0.0.0.0:18082
      - --advertise-pandaproxy-addr=internal://redpanda:8082,external://localhost:18082
      - --overprovisioned
      - --smp=1
      - --memory=2G
      - --reserve-memory=200M
      - --node-id=0
      - --check=false
  clickhousedb:
    image: docker.io/clickhouse/clickhouse-server:${CLICKHOUSE_VERSION:-latest}
    volumes:
      - .clickhouse/configs/scripts:/docker-entrypoint-initdb.d
      - .clickhouse/data:/var/lib/clickhouse/
      - .clickhouse/logs:/var/log/clickhouse-server/
      - .clickhouse/configs/users:/etc/clickhouse-server/users.d
    environment:
      - CLICKHOUSE_DB=${DB_NAME:-local}
      - CLICKHOUSE_USER=${CLICKHOUSE_USER:-panda}
      - CLICKHOUSE_PASSWORD=${CLICKHOUSE_PASSWORD:-pandapass}
      - CLICKHOUSE_DEFAULT_ACCESS_MANAGEMENT=1
    ports:
      - "${CLICKHOUSE_HOST_PORT:-18123}:8123"
      - "${CLICKHOUSE_POSTGRES_PORT:-9005}:9005"
    ulimits:
      nofile:
        soft: 20000
        hard: 40000
  console:
    image: docker.io/514labs/moose-console:${CONSOLE_VERSION:-latest}
    environment:
      - CLICKHOUSE_DB=${DB_NAME:-local}
      - CLICKHOUSE_USER=${CLICKHOUSE_USER:-panda}
      - CLICKHOUSE_PASSWORD=${CLICKHOUSE_PASSWORD:-pandapass}
      - CLICKHOUSE_HOST=clickhousedb
      - CLICKHOUSE_PORT=8123
    ports:
      - "${CONSOLE_HOST_PORT:-3001}:3000"
"#;

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

pub fn stop_containers(project: &Project) -> std::io::Result<String> {
    let child = compose_command(project)
        .arg("down")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;

    output_to_result(output)
}

fn compose_command(project: &Project) -> Command {
    let mut command = Command::new("docker");

    command
        .arg("compose")
        .arg("-f")
        .arg(project.internal_dir().unwrap().join("docker-compose.yml"))
        .arg("-p")
        .arg(project.name());
    command
}

pub fn start_containers(project: &Project) -> std::io::Result<String> {
    let console_version = if cfg!(debug_assertions) {
        "latest"
    } else {
        CLI_VERSION
    };

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
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;

    output_to_result(output)
}

pub fn create_compose_file(project: &Project) -> std::io::Result<()> {
    let compose_file = project.internal_dir()?.join("docker-compose.yml");

    std::fs::write(compose_file, COMPOSE_FILE)
}

pub fn run_rpk_cluster_info(project_name: &str) -> std::io::Result<String> {
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

    output_to_result(output)
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

fn output_to_result(output: std::process::Output) -> std::io::Result<String> {
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}
