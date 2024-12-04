use log::{error, info};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use crate::framework::blocks::model::BlocksError;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;

use super::bin;

const BLOCKS_RUNNER_BIN: &str = "blocks";

// TODO: Abstract away ClickhouseConfig to support other databases
// TODO: Bubble up compilation errors to the user
pub fn run(
    clickhouse_config: ClickHouseConfig,
    blocks_path: &Path,
    project_path: &Path,
) -> Result<Child, BlocksError> {
    let host_port = clickhouse_config.host_port.to_string();
    let use_ssl = clickhouse_config.use_ssl.to_string();
    let args = vec![
        blocks_path.to_str().unwrap(),
        &clickhouse_config.db_name,
        &clickhouse_config.host,
        &host_port,
        &clickhouse_config.user,
        &clickhouse_config.password,
        &use_ssl,
    ];

    let mut blocks_process = bin::run(BLOCKS_RUNNER_BIN, project_path, &args)?;

    let stdout = blocks_process
        .stdout
        .take()
        .expect("Blocks process did not have a handle to stdout");

    let stderr = blocks_process
        .stderr
        .take()
        .expect("Blocks process did not have a handle to stderr");

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

    Ok(blocks_process)
}
