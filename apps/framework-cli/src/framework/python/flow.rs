use std::path::Path;

use crate::infrastructure::stream::redpanda::RedpandaConfig;

use super::executor;

fn run(redpanda_config: RedpandaConfig, source_topic: &str, target_topic: &str, flow_path: &Path) {
    let output = executor::run_python_program(executor::PythonProgram::FlowRunner {
        args: vec![
            source_topic.to_string(),
            target_topic.to_string(),
            flow_path.to_str().unwrap().to_string(),
            redpanda_config.broker,
        ],
    });

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);
}

// tests
#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    #[ignore]
    fn test_run() {
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

        run(redpanda_config, source_topic, target_topic, flow_path);
    }

    #[test]
    #[ignore]
    fn test_run_with_invalid_flow_file() {
        let redpanda_config = RedpandaConfig::default();
        let source_topic = "source";
        let target_topic = "target";
        let flow_path = Path::new(
            "/Users/timdelisle/Dev/igloo-stack/apps/framework-cli/tests/python/flows/invalid",
        );

        run(redpanda_config, source_topic, target_topic, flow_path);
    }
}
