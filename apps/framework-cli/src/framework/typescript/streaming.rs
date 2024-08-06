use log::{error, info};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use crate::infrastructure::stream::redpanda::RedpandaConfig;

use super::ts_node;

const FUNCTION_RUNNER_WRAPPER: &str = include_str!("ts_scripts/streaming-function.ts");

// TODO: we currently refer redpanda configuration here. If we want to be able to
// abstract this to other type of streaming engine, we will need to be able to abstract this away.
pub fn run(
    redpanda_config: &RedpandaConfig,
    source_topic: &str,
    target_topic: &str,
    target_topic_config: &str,
    streaming_function_file: &Path,
    // TODO Remove the anyhow type here
) -> Result<Child, std::io::Error> {
    let mut args = vec![
        source_topic,
        target_topic,
        target_topic_config,
        streaming_function_file.to_str().unwrap(),
        &redpanda_config.broker,
    ];

    info!(
        "Starting a streaming function with the following public arguments: {:#?}",
        args
    );

    if redpanda_config.sasl_username.is_some() {
        args.push(redpanda_config.sasl_username.as_ref().unwrap());
    }

    if redpanda_config.sasl_password.is_some() {
        args.push(redpanda_config.sasl_password.as_ref().unwrap());
    }

    if redpanda_config.sasl_mechanism.is_some() {
        args.push(redpanda_config.sasl_mechanism.as_ref().unwrap());
    }

    if redpanda_config.security_protocol.is_some() {
        args.push(redpanda_config.security_protocol.as_ref().unwrap());
    }

    let mut streaming_function_process = ts_node::run(FUNCTION_RUNNER_WRAPPER, &args)?;

    let stdout = streaming_function_process
        .stdout
        .take()
        .expect("Streaming process did not have a handle to stdout");

    let stderr = streaming_function_process
        .stderr
        .take()
        .expect("Streaming process did not have a handle to stderr");

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

    Ok(streaming_function_process)
}
