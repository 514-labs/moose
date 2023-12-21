use std::{path::PathBuf, process::Command};

use crate::constants::PANDA_NETWORK;
use crate::infrastructure::olap::clickhouse::config::ClickhouseConfig;

fn network_command(command: &str, network_name: &str) -> std::io::Result<std::process::Output> {
    Command::new("docker")
        .arg("network")
        .arg(command)
        .arg(network_name)
        .output()
}

pub fn network_list() -> std::io::Result<std::process::Output> {
    Command::new("docker").arg("network").arg("ls").output()
}

pub fn remove_network(network_name: &str) -> std::io::Result<std::process::Output> {
    network_command("rm", network_name)
}

pub fn create_network(network_name: &str) -> std::io::Result<std::process::Output> {
    network_command("create", network_name)
}

pub fn stop_container(name: &str) -> std::io::Result<std::process::Output> {
    Command::new("docker").arg("stop").arg(name).output()
}

pub fn filter_list_containers(name: &str) -> std::io::Result<std::process::Output> {
    Command::new("docker")
        .arg("ps")
        .arg("--filter")
        .arg("name=".to_owned() + name)
        .output()
}

pub fn run_rpk_cluster_info() -> std::io::Result<std::process::Output> {
    Command::new("docker")
        .arg("exec")
        .arg("redpanda-1")
        .arg("rpk")
        .arg("cluster")
        .arg("info")
        .output()
}

pub fn run_rpk_command(args: Vec<String>) -> std::io::Result<std::process::Output> {
    Command::new("docker")
        .arg("exec")
        .arg("redpanda-1")
        .arg("rpk")
        .args(args)
        .output()
}

pub fn run_red_panda(igloo_dir: PathBuf) -> std::io::Result<std::process::Output> {
    let mount_dir = igloo_dir.join(".panda_house");

    Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--pull=always")
        .arg("--name=redpanda-1")
        .arg("--rm")
        .arg(format!("--network={PANDA_NETWORK}"))
        .arg("--volume=".to_owned() + mount_dir.to_str().unwrap() + ":/tmp/panda_house")
        .arg("--publish=9092:9092")
        .arg("--publish=19092:19092")
        .arg("--publish=9644:9644")
        .arg("docker.redpanda.com/redpandadata/redpanda:latest")
        .arg("redpanda")
        .arg("start")
        .arg("--kafka-addr internal://0.0.0.0:9092,external://0.0.0.0:19092")
        .arg(format!(
            "--advertise-kafka-addr internal://{}:9092,external://localhost:19092",
            "redpanda-1"
        ))
        .arg("--pandaproxy-addr internal://0.0.0.0:8082,external://0.0.0.0:18082")
        .arg(format!(
            "--advertise-pandaproxy-addr internal://{}:8082,external://localhost:18082",
            "redpanda-1"
        ))
        .arg("--overprovisioned")
        .arg("--smp 1")
        .arg("--memory 2G")
        .arg("--reserve-memory 200M")
        .arg("--node-id 0")
        .arg("--check=false")
        .output()
}

pub fn run_clickhouse(
    igloo_dir: PathBuf,
    config: ClickhouseConfig,
) -> std::io::Result<std::process::Output> {
    let data_mount_dir = igloo_dir.join(".clickhouse/data");
    let logs_mount_dir = igloo_dir.join(".clickhouse/logs");
    // let server_config_mount_dir = igloo_dir.join(".clickhouse/configs/server");
    let user_config_mount_dir = igloo_dir.join(".clickhouse/configs/users");
    let scripts_config_mount_dir = igloo_dir.join(".clickhouse/configs/scripts");

    // TODO: Make this configurable by the user
    // Specifying the user and password in plain text here. This should be a user input
    // Double check the access management flag and why it needs to be set to 1
    Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--pull=always")
        .arg("--name=clickhousedb-1")
        .arg("--rm")
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
        .output()
}
