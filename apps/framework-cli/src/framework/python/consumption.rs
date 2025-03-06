use crate::framework::consumption::model::ConsumptionQueryParam;
use crate::framework::python::executor::{run_python_program, PythonProgram};
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::processes::consumption_registry::ConsumptionError;
use crate::project::JwtConfig;
use log::{error, info};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use super::executor;

pub fn run(
    clickhouse_config: ClickHouseConfig,
    jwt_config: Option<JwtConfig>,
    consumption_path: &Path,
    temporal_url: &str,
) -> Result<Child, ConsumptionError> {
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
        consumption_path.to_str().unwrap().to_string(),
        clickhouse_config.db_name,
        clickhouse_config.host,
        clickhouse_config.host_port.to_string(),
        clickhouse_config.user,
        clickhouse_config.password,
        clickhouse_config.use_ssl.to_string(),
        jwt_secret,
        jwt_issuer,
        jwt_audience,
        enforce_on_all_consumptions_apis,
        temporal_url.to_string(),
    ];

    let mut consumption_process =
        executor::run_python_program(executor::PythonProgram::ConsumptionRunner { args })?;

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

pub async fn load_python_query_param(
    path: &Path,
) -> Result<Vec<ConsumptionQueryParam>, std::io::Error> {
    let args = vec![path.file_name().unwrap().to_str().unwrap().to_string()];
    let process = run_python_program(PythonProgram::LoadApiParam { args })?;
    let output = process.wait_with_output().await?;

    if !output.status.success() {
        return Err(std::io::Error::other(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    let raw_string_stdout = String::from_utf8_lossy(&output.stdout);

    let config = serde_json::from_str::<crate::framework::consumption::loader::QueryParamOutput>(
        &raw_string_stdout,
    )
    .map_err(std::io::Error::other)?;
    Ok(config.params)
}
