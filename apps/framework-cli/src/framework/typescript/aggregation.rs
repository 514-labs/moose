use log::{error, info};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use crate::framework::aggregations::model::AggregationError;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;

use super::bin;

const BLOCKS_RUNNER_BIN: &str = "blocks";
const AGGREGATIONS_RUNNER_BIN: &str = "aggregations";

// TODO: Abstract away ClickhouseConfig to support other databases
// TODO: Bubble up compilation errors to the user
pub fn run(
    clickhouse_config: ClickHouseConfig,
    aggregations_path: &Path,
    is_blocks: bool,
    project_path: &Path,
) -> Result<Child, AggregationError> {
    let host_port = clickhouse_config.host_port.to_string();
    let use_ssl = clickhouse_config.use_ssl.to_string();
    let args = vec![
        aggregations_path.to_str().unwrap(),
        &clickhouse_config.db_name,
        &clickhouse_config.host,
        &host_port,
        &clickhouse_config.user,
        &clickhouse_config.password,
        &use_ssl,
    ];

    let mut aggregation_process = if is_blocks {
        bin::run(BLOCKS_RUNNER_BIN, project_path, &args)?
    } else {
        bin::run(AGGREGATIONS_RUNNER_BIN, project_path, &args)?
    };

    let stdout = aggregation_process
        .stdout
        .take()
        .expect("Aggregation process did not have a handle to stdout");

    let stderr = aggregation_process
        .stderr
        .take()
        .expect("Aggregation process did not have a handle to stderr");

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

    Ok(aggregation_process)
}
