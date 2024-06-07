use std::path::Path;

use tokio::process::Child;

use crate::infrastructure::stream::redpanda::RedpandaConfig;
use tokio::io::AsyncBufReadExt;

use super::executor;

pub fn run(
    redpanda_config: RedpandaConfig,
    source_topic: &str,
    target_topic: &str,
    flow_path: &Path,
) -> Result<Child, std::io::Error> {
    let mut flow_process = executor::run_python_program(executor::PythonProgram::FlowRunner {
        args: vec![
            source_topic.to_string(),
            target_topic.to_string(),
            flow_path.to_str().unwrap().to_string(),
            redpanda_config.broker,
        ],
    })?;

    let stdout = flow_process
        .stdout
        .take()
        .expect("Flow process did not have a handle to stdout");
    let stderr = flow_process
        .stderr
        .take()
        .expect("Flow process did not have a handle to stderr");

    let mut stdout_reader = tokio::io::BufReader::new(stdout).lines();
    let mut stderr_reader = tokio::io::BufReader::new(stderr).lines();

    tokio::spawn(async move {
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            log::info!("{}", line);
        }
    });

    tokio::spawn(async move {
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            log::error!("{}", line);
        }
    });

    Ok(flow_process)
}

// tests
#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_run() {
        // Use these tests by configuring a timeout in the flow runner
        //  consumer = KafkaConsumer(
        //     source_topic,
        //     client_id= "python_flow_consumer",
        //     group_id=flow_id,
        //     bootstrap_servers=broker,
        // +   consumer_timeout_ms=10000,
        //     value_deserializer=lambda m: json.loads(m.decode('utf-8'))
        // )
        // You can then run a moose instance and send data to the endpoints
        let redpanda_config = RedpandaConfig::default();
        let source_topic = "UserActivity_0_0";
        let target_topic = "ParsedActivity_0_0";
        let flow_path = Path::new(
            "/Users/timdelisle/Dev/igloo-stack/apps/framework-cli/tests/python/flows/valid",
        );

        let child = run(redpanda_config, source_topic, target_topic, flow_path).unwrap();

        let output = child.wait_with_output().await.unwrap();

        //print output stdout and stderr
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    #[tokio::test]
    #[ignore]
    async fn test_run_with_invalid_flow_file() {
        let redpanda_config = RedpandaConfig::default();
        let source_topic = "source";
        let target_topic = "target";
        let flow_path = Path::new(
            "/Users/timdelisle/Dev/igloo-stack/apps/framework-cli/tests/python/flows/invalid",
        );

        let child = run(redpanda_config, source_topic, target_topic, flow_path).unwrap();

        let output = child.wait_with_output().await.unwrap();

        //print output stdout and stderr
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}
