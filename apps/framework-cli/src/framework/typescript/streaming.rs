use log::{error, info};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;

use crate::infrastructure::stream::redpanda::models::RedpandaConfig;
use crate::infrastructure::stream::StreamConfig;

use super::bin;

const FUNCTION_RUNNER_BIN: &str = "streaming-functions";

// TODO: we currently refer redpanda configuration here. If we want to be able to
// abstract this to other type of streaming engine, we will need to be able to abstract this away.
#[allow(clippy::too_many_arguments)]
pub fn run(
    redpanda_config: &RedpandaConfig,
    source_topic: &StreamConfig,
    target_topic: &StreamConfig,
    streaming_function_file: &Path,
    project_path: &Path,
    max_subscriber_count: usize,
    is_dmv2: bool,
    // TODO Remove the anyhow type here
) -> Result<Child, std::io::Error> {
    let subscriber_count_str = max_subscriber_count.to_string();
    let is_dmv2_str = is_dmv2.to_string();

    let source_topic_config_str = source_topic.as_json_string();
    let target_topic_config_str = target_topic.as_json_string();

    let mut args: Vec<&str> = vec![
        source_topic_config_str.as_str(),
        target_topic_config_str.as_str(),
        streaming_function_file.to_str().unwrap(),
        &redpanda_config.broker,
        &subscriber_count_str,
        &is_dmv2_str,
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

    let mut streaming_function_process = bin::run(FUNCTION_RUNNER_BIN, project_path, &args)?;

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
