use log::{error, info};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use crate::framework::consumption::model::ConsumptionError;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;

use super::ts_node;

const CONSUMPTION_RUNNER_WRAPPER: &str = include_str!("ts_scripts/consumption-api.ts");

// TODO: Abstract away ClickhouseConfig to support other databases
// TODO: Bubble up compilation errors to the user
pub fn run(
    clickhouse_config: ClickHouseConfig,
    consumption_path: &Path,
) -> Result<Child, ConsumptionError> {
    let host_port = clickhouse_config.host_port.to_string();
    let use_ssl = clickhouse_config.use_ssl.to_string();
    let args = vec![
        consumption_path.to_str().unwrap(),
        &clickhouse_config.db_name,
        &clickhouse_config.host,
        &host_port,
        &clickhouse_config.user,
        &clickhouse_config.password,
        &use_ssl,
    ];

    let mut consumption_process = ts_node::run(CONSUMPTION_RUNNER_WRAPPER, &args)?;

    let stdout = consumption_process
        .stdout
        .take()
        .expect("Consumption process did not have a handle to stdout");

    let stderr = consumption_process
        .stderr
        .take()
        .expect("Consumption process did not have a handle to stderr");

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

    Ok(consumption_process)
}
