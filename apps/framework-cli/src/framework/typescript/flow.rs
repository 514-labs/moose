use log::{error, info};
use std::io::BufRead;
use std::process::Child;
use std::{io::BufReader, path::Path};

use crate::infrastructure::stream::redpanda::RedpandaConfig;

use super::ts_node;

const FLOW_RUNNER_WRAPPER: &str = include_str!("ts_scripts/flow.ts");

// TODO: we currently refer repanda configuration here. If we want to be able to
// abstract this to other type of streaming engine, we will need to be able to abstract this away.
// TODO: compilation errors are not proxied to the user in dev mode. We need to fix it
// so that they can have some feedback when they mess up the typescript
pub fn run(
    redpanda_config: RedpandaConfig,
    source_topic: &str,
    target_topic: &str,
    flow_path: &Path,
    // TODO Remove the anyhow type here
) -> Result<Child, std::io::Error> {
    let mut args = vec![
        source_topic,
        target_topic,
        flow_path.to_str().unwrap(),
        &redpanda_config.broker,
    ];

    info!(
        "Starting a flow with the following public arguments: {:#?}",
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

    let mut flow_process = ts_node::run(FLOW_RUNNER_WRAPPER, &args)?;

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
        while let Some(Ok(line)) = stdout_reader.next() {
            info!("{}", line);
        }
    });

    tokio::spawn(async move {
        while let Some(Ok(line)) = stderr_reader.next() {
            error!("{}", line);
        }
    });

    Ok(flow_process)
}
