use log::{error, info};
use std::{path::Path, vec};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::infrastructure::stream::redpanda::RedpandaConfig;

use super::ts_node::run;

const FLOW_RUNNER_WRAPPER: &str = include_str!("ts_scripts/flow.ts");

// TODO: we currently refer repanda configuration here. If we want to be able to
// abstract this to other type of streaming engine, we will need to be able to abstract this away.
fn run_flow(
    redpanda_config: RedpandaConfig,
    source_topic: &str,
    target_topic: &str,
    flow_path: &Path,
) {
    let mut args = vec![
        source_topic,
        target_topic,
        flow_path.to_str().unwrap(),
        &redpanda_config.broker,
    ];

    if redpanda_config.sasl_username.is_some() {
        args.push(&redpanda_config.sasl_username.as_ref().unwrap());
    }

    if redpanda_config.sasl_password.is_some() {
        args.push(&redpanda_config.sasl_password.as_ref().unwrap());
    }

    if redpanda_config.sasl_mechanism.is_some() {
        args.push(&redpanda_config.sasl_mechanism.as_ref().unwrap());
    }

    if redpanda_config.security_protocol.is_some() {
        args.push(&redpanda_config.security_protocol.as_ref().unwrap());
    }

    let flow_process = run(FLOW_RUNNER_WRAPPER, &args)?;

    let stdout = flow_process
        .stdout
        .take()
        .expect("Deno process did not have a handle to stdout");

    let stderr = flow_process
        .stderr
        .take()
        .expect("Deno process did not have a handle to stderr");

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

    Ok(())
}

struct Flow {
    source_topic: String,
    target_topic: String,
    flow_path: Path,
}

fn get_all_current_flows() -> Vec<Flow> {}
