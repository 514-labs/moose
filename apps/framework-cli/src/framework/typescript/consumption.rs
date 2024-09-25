use log::{error, info};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::processes::consumption_registry::ConsumptionError;
use crate::project::JwtConfig;

use super::bin;

const CONSUMPTION_RUNNER_BIN: &str = "consumption-apis";

// TODO: Abstract away ClickhouseConfig to support other databases
// TODO: Bubble up compilation errors to the user
pub fn run(
    clickhouse_config: ClickHouseConfig,
    jwt_config: Option<JwtConfig>,
    consumption_path: &Path,
    project_path: &Path,
) -> Result<Child, ConsumptionError> {
    let host_port = clickhouse_config.host_port.to_string();
    let use_ssl = clickhouse_config.use_ssl.to_string();

    let jwt_secret = jwt_config
        .as_ref()
        .map(|jwt| jwt.secret.clone())
        .unwrap_or("".to_string());

    let jwt_issuer = jwt_config
        .as_ref()
        .map(|jwt| jwt.issuer.clone())
        .unwrap_or("".to_string());

    let jwt_audience = jwt_config
        .as_ref()
        .map(|jwt| jwt.audience.clone())
        .unwrap_or("".to_string());

    let enforce_on_all_consumptions_apis = jwt_config
        .as_ref()
        .map(|jwt| jwt.enforce_on_all_consumptions_apis.to_string())
        .unwrap_or("false".to_string());

    let args = vec![
        consumption_path.to_str().unwrap(),
        &clickhouse_config.db_name,
        &clickhouse_config.host,
        &host_port,
        &clickhouse_config.user,
        &clickhouse_config.password,
        &use_ssl,
        &jwt_secret,
        &jwt_issuer,
        &jwt_audience,
        &enforce_on_all_consumptions_apis,
    ];

    let mut consumption_process = bin::run(CONSUMPTION_RUNNER_BIN, project_path, &args)?;

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
