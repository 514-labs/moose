use std::{process::Command, path::PathBuf};

fn network_command(command: &str) -> std::io::Result<std::process::Output>{
    Command::new("docker")
        .arg("network")
        .arg(command)
        .arg("panda-house")
        .output()
}

pub fn network_list() -> std::io::Result<std::process::Output>{
    Command::new("docker")
        .arg("network")
        .arg("ls")
        .output()
}

pub fn remove_network() -> std::io::Result<std::process::Output>{
    network_command("rm")
}

pub fn create_network() -> std::io::Result<std::process::Output>{
    network_command("create")
}

pub fn stop_container(name: &str) -> std::io::Result<std::process::Output>{
    Command::new("docker")
        .arg("stop")
        .arg(name)
        .output()
}

pub fn filter_list_containers(name: &str) -> std::io::Result<std::process::Output>{
    Command::new("docker")
        .arg("ps")
        .arg("--filter")
        .arg("name=".to_owned() + name)
        .output()
}

pub fn run_rpk_list() -> std::io::Result<std::process::Output>{
    Command::new("docker")
        .arg("exec")
        .arg("redpanda-1")
        .arg("rpk")
        .arg("cluster")
        .arg("info")
        .output()
}

pub fn run_red_panda(igloo_dir:  PathBuf) -> std::io::Result<std::process::Output>{
    let mount_dir = igloo_dir.join(".panda_house");

    Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--pull=always")
        .arg("--name=redpanda-1")
        .arg("--rm")
        .arg("--network=panda-house")
        .arg("--volume=".to_owned() + mount_dir.to_str().unwrap() + ":/tmp/panda_house")
        .arg("--publish=9092:9092")
        .arg("--publish=9644:9644")
        .arg("docker.vectorized.io/vectorized/redpanda:latest")
        .arg("redpanda")
        .arg("start")
        .arg("--advertise-kafka-addr redpanda-1")
        .arg("--overprovisioned")
        .arg("--smp 1")
        .arg("--memory 2G")
        .arg("--reserve-memory 200M")
        .arg("--node-id 0")
        .arg("--check=false")
        .output()
}

pub fn run_clickhouse(igloo_dir: PathBuf) -> std::io::Result<std::process::Output> {
    let data_mount_dir = igloo_dir.join(".clickhouse/data");
    let logs_mount_dir = igloo_dir.join(".clickhouse/logs");
    let config_mount_dir = igloo_dir.join(".clickhouse/configs");
    
    // TODO: Make this configurable by the user
    // Specifying the user and password in plain text here. This should be a user input
    // Double check the access management flag and why it needs to be set to 1
    Command::new("docker")
        .arg("run")
        .arg("-d")
        .arg("--pull=always")
        .arg("--name=clickhousedb-1")
        .arg("--rm")
        .arg("--volume=".to_owned() + data_mount_dir.to_str().unwrap() + ":/var/lib/clickhouse/")
        .arg("--volume=".to_owned() + logs_mount_dir.to_str().unwrap() + ":/var/log/clickhouse-server/")
        .arg("--volume=".to_owned() + config_mount_dir.to_str().unwrap() + ":/etc/clickhouse-server/config.d/*.xml")
        .arg("--env=CLICKHOUSE_DB=panda_house")
        .arg("--env=CLICKHOUSE_USER=panda")
        .arg("--env=CLICKHOUSE_DEFAULT_ACCESS_MANAGEMENT=1") // Might be unsafe
        .arg("--env=CLICKHOUSE_PASSWORD=pandapass")
        .arg("--network=panda-house")
        .arg("--publish=18123:8123")
        .arg("--publish=9005:9005")
        .arg("--ulimit=nofile=262144:262144")
        .arg("docker.io/clickhouse/clickhouse-server")
        .output()

}