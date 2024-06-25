use log::{error, info};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use crate::framework::consumption::model::ConsumptionError;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;

use super::executor;

pub fn run(
    clickhouse_config: ClickHouseConfig,
    consumption_path: &Path,
) -> Result<Child, ConsumptionError> {
    let args = vec![
        consumption_path.to_str().unwrap().to_string(),
        clickhouse_config.db_name,
        clickhouse_config.host,
        clickhouse_config.host_port.to_string(),
        clickhouse_config.user,
        clickhouse_config.password,
        clickhouse_config.use_ssl.to_string(),
    ];

    let mut aggregation_process =
        executor::run_python_program(executor::PythonProgram::ConsumptionRunner { args })?;

    let stdout = aggregation_process
        .stdout
        .take()
        .expect("Consumption process did not have a handle to stdout");

    let stderr = aggregation_process
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

    Ok(aggregation_process)
}
