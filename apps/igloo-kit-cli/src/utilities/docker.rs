use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use crate::infrastructure::console::ConsoleConfig;
use crate::infrastructure::olap::clickhouse::config::ClickhouseConfig;
use crate::utilities::constants::{
    CLICKHOUSE_CONTAINER_NAME, CLI_VERSION, CONSOLE_CONTAINER_NAME, PANDA_NETWORK,
    REDPANDA_CONTAINER_NAME,
};
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, from_str};

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

fn network_command(command: &str, network_name: &str) -> std::io::Result<String> {
    let child = Command::new("docker")
        .arg("network")
        .arg(command)
        .arg(network_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        // match std error with a regex and a match statement if it contains network and already exists
        let owned = String::from_utf8_lossy(&output.stderr).into_owned();
        let std_error_str = owned.as_str();

        match std_error_str {
            _ if std_error_str.contains("network") && std_error_str.contains("already exists") => {
                Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    String::from_utf8_lossy(&output.stderr),
                ))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            )),
        }
    }
}

pub fn network_list() -> std::io::Result<Vec<NetworkRow>> {
    let child = Command::new("docker")
        .arg("network")
        .arg("ls")
        .arg("--format")
        .arg("json")
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
    let networks: Vec<NetworkRow> = output_str
        .split('\n')
        .filter(|line| !line.is_empty())
        .map(|line| from_str(line).expect("Failed to parse network row"))
        .collect();

    Ok(networks)
}

pub fn remove_network(network_name: &str) -> std::io::Result<String> {
    network_command("rm", network_name)
}

pub fn create_network(network_name: &str) -> std::io::Result<String> {
    network_command("create", network_name)
}

pub fn stop_container(name: &str) -> std::io::Result<String> {
    let child = Command::new("docker")
        .arg("stop")
        .arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let output = child.wait_with_output()?;

    output_to_result(output)
}

pub fn run_rpk_cluster_info() -> std::io::Result<String> {
    let child = Command::new("docker")
        .arg("exec")
        .arg("redpanda-1")
        .arg("rpk")
        .arg("cluster")
        .arg("info")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    output_to_result(output)
}

pub fn run_rpk_command(args: Vec<String>) -> std::io::Result<String> {
    let child = Command::new("docker")
        .arg("exec")
        .arg("redpanda-1")
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

pub fn safe_start_redpanda_container(igloo_dir: PathBuf) -> std::io::Result<String> {
    //! Starts a redpanda container if it is not already running. If the doesn't exist, it will be created.
    //!
    //! # Arguments
    //!
    //! * `igloo_dir` - The path to the igloo directory

    match start_redpanda_container() {
        Ok(output) => Ok(output),
        Err(_) => run_red_panda(igloo_dir),
    }
}

fn run_red_panda(igloo_dir: PathBuf) -> std::io::Result<String> {
    let mount_dir = igloo_dir.join(".panda_house");

    let child = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--pull=always")
        .arg(format!("--name={REDPANDA_CONTAINER_NAME}"))
        // .arg("--rm")
        .arg(format!("--network={PANDA_NETWORK}"))
        .arg("--volume=".to_owned() + mount_dir.to_str().unwrap() + ":/tmp/panda_house")
        .arg("--publish=9092:9092")
        .arg("--publish=19092:19092")
        .arg("--publish=9644:9644")
        .arg("docker.redpanda.com/redpandadata/redpanda:latest")
        .arg("redpanda")
        .arg("start")
        .arg("--kafka-addr=internal://0.0.0.0:9092,external://0.0.0.0:19092")
        .arg(format!(
            "--advertise-kafka-addr=internal://{}:9092,external://localhost:19092",
            "redpanda-1"
        ))
        .arg("--pandaproxy-addr=internal://0.0.0.0:8082,external://0.0.0.0:18082")
        .arg(format!(
            "--advertise-pandaproxy-addr=internal://{}:8082,external://localhost:18082",
            "redpanda-1"
        ))
        .arg("--overprovisioned")
        .arg("--smp=1")
        .arg("--memory=2G")
        .arg("--reserve-memory=200M")
        .arg("--node-id=0")
        .arg("--check=false")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    output_to_result(output)
}

pub fn safe_start_clickhouse_container(
    igloo_dir: PathBuf,
    config: ClickhouseConfig,
) -> std::io::Result<String> {
    //! Starts a clickhouse container if it is not already running. If the doesn't exist, it will be created.
    //!
    //! # Arguments
    //!
    //! * `igloo_dir` - The path to the igloo directory
    //! * `config` - The clickhouse configuration

    match start_clickhouse_container() {
        Ok(output) => Ok(output),
        Err(_) => run_clickhouse(igloo_dir, config),
    }
}

fn run_clickhouse(igloo_dir: PathBuf, config: ClickhouseConfig) -> std::io::Result<String> {
    let data_mount_dir = igloo_dir.join(".clickhouse/data");
    let logs_mount_dir = igloo_dir.join(".clickhouse/logs");
    // let server_config_mount_dir = igloo_dir.join(".clickhouse/configs/server");
    let user_config_mount_dir = igloo_dir.join(".clickhouse/configs/users");
    let scripts_config_mount_dir = igloo_dir.join(".clickhouse/configs/scripts");

    // TODO: Make this configurable by the user
    // Specifying the user and password in plain text here. This should be a user input
    // Double check the access management flag and why it needs to be set to 1
    let child = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--pull=always")
        .arg(format!("--name={CLICKHOUSE_CONTAINER_NAME}"))
        // .arg("--rm")
        .arg(
            "--volume=".to_owned()
                + scripts_config_mount_dir.to_str().unwrap()
                + ":/docker-entrypoint-initdb.d",
        )
        .arg("--volume=".to_owned() + data_mount_dir.to_str().unwrap() + ":/var/lib/clickhouse/")
        .arg(
            "--volume=".to_owned()
                + logs_mount_dir.to_str().unwrap()
                + ":/var/log/clickhouse-server/",
        )
        // .arg("--volume=".to_owned() + server_config_mount_dir.to_str().unwrap() + ":/etc/clickhouse-server/config.d")
        .arg(
            "--volume=".to_owned()
                + user_config_mount_dir.to_str().unwrap()
                + ":/etc/clickhouse-server/users.d",
        )
        .arg(format!("--env=CLICKHOUSE_DB={}", config.db_name))
        .arg(format!("--env=CLICKHOUSE_USER={}", config.user))
        .arg("--env=CLICKHOUSE_DEFAULT_ACCESS_MANAGEMENT=1") // Might be unsafe
        .arg(format!("--env=CLICKHOUSE_PASSWORD={}", config.password))
        .arg(format!("--network={PANDA_NETWORK}"))
        .arg(format!("--publish={}:8123", config.host_port))
        .arg(format!("--publish={}:9005", config.postgres_port))
        .arg("--ulimit=nofile=262144:262144")
        .arg("docker.io/clickhouse/clickhouse-server")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    output_to_result(output)
}

pub fn safe_start_console_container(
    console_config: &ConsoleConfig,
    clickhouse_config: &ClickhouseConfig,
) -> std::io::Result<String> {
    //! Starts a console container if it is not already running. If the doesn't exist, it will be created.

    match start_console_container() {
        Ok(output) => Ok(output),
        Err(_) => run_console(console_config, clickhouse_config),
    }
}

fn run_console(
    console_config: &ConsoleConfig,
    clickhouse_config: &ClickhouseConfig,
) -> std::io::Result<String> {
    let child = Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg(format!("--name={CONSOLE_CONTAINER_NAME}"))
        .arg(format!("--env=CLICKHOUSE_DB={}", clickhouse_config.db_name))
        .arg(format!("--env=CLICKHOUSE_USER={}", clickhouse_config.user))
        .arg(format!(
            "--env=CLICKHOUSE_PASSWORD={}",
            clickhouse_config.password
        ))
        .arg(format!("--network={PANDA_NETWORK}"))
        .arg(format!("--publish={}:3000", console_config.host_port))
        .arg(format!("docker.io/514labs/console:{CLI_VERSION}"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    output_to_result(output)
}

fn start_redpanda_container() -> std::io::Result<String> {
    start_container("redpanda-1")
}

fn start_clickhouse_container() -> std::io::Result<String> {
    start_container("clickhousedb-1")
}

fn start_console_container() -> std::io::Result<String> {
    start_container("console-1")
}

fn start_container(name: &str) -> std::io::Result<String> {
    let child = Command::new("docker")
        .arg("start")
        .arg(name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let output = child.wait_with_output()?;

    output_to_result(output)
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
