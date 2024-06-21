//! # Executes Python code in a subprocess.
//! This module provides a Python executor that can run Python code in a subprocess

use std::process::Stdio;

use tokio::process::{Child, Command};

/// Checks if the Python interpreter is available

/// Checks the version of the Python interpreter

/// Ensures that Python3.7 is available on the system

pub enum PythonSerializers {
    FrameworkObjectSerializer,
    ProjectObjectSerializer,
}

impl PythonSerializers {
    pub fn get_path(&self) -> &str {
        match self {
            PythonSerializers::FrameworkObjectSerializer => {
                "src/framework/python/scripts/framework_object_serializer.py"
            }
            PythonSerializers::ProjectObjectSerializer => {
                "src/framework/python/scripts/project_object_serializer.py"
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PythonProgram {
    FlowRunner { args: Vec<String> },
    AggregationsRunner { args: Vec<String> },
    ConsumptionRunner { args: Vec<String> },
}

pub static FLOW_RUNNER: &str = include_str!("scripts/flow_runner.py");
pub static AGGREGATIONS_RUNNER: &str = include_str!("scripts/aggregations_runner.py");
pub static CONSUMPTION_RUNNER: &str = include_str!("scripts/consumption_runner.py");

/// Executes a Python program in a subprocess
pub fn run_python_program(program: PythonProgram) -> Result<Child, std::io::Error> {
    let get_args = match program.clone() {
        PythonProgram::FlowRunner { args } => args,
        PythonProgram::AggregationsRunner { args } => args,
        PythonProgram::ConsumptionRunner { args } => args,
    };

    let program_string = match program {
        PythonProgram::FlowRunner { .. } => FLOW_RUNNER,
        PythonProgram::AggregationsRunner { .. } => AGGREGATIONS_RUNNER,
        PythonProgram::ConsumptionRunner { .. } => CONSUMPTION_RUNNER,
    };

    Command::new("python3")
        .arg("-c")
        .arg(program_string)
        .args(get_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

// TESTs
#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::infrastructure::{
        olap::clickhouse::config::ClickHouseConfig, stream::redpanda::RedpandaConfig,
    };

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_run_python_flow_runner_program() {
        let redpanda_config = RedpandaConfig::default();
        let source_topic = "UserActivity_0_0";
        let target_topic = "ParsedActivity_0_0";
        let flow_path = Path::new(
            "/Users/timdelisle/Dev/igloo-stack/apps/framework-cli/tests/python/flows/valid",
        );

        let program = PythonProgram::FlowRunner {
            args: vec![
                source_topic.to_string(),
                target_topic.to_string(),
                flow_path.to_str().unwrap().to_string(),
                redpanda_config.broker,
            ],
        };

        let child = run_python_program(program).unwrap();
        let output = child.wait_with_output().await.unwrap();

        //print output stdout and stderr
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    #[tokio::test]
    #[ignore]
    async fn test_run_python_aggregations_runner_program() {
        let agg_path = Path::new(
            "/Users/timdelisle/Dev/igloo-stack/apps/framework-cli/tests/python/aggregations/valid",
        );
        let clickhouse_config = ClickHouseConfig::default();

        let program = PythonProgram::AggregationsRunner {
            args: vec![
                agg_path.to_str().unwrap().to_string(),
                clickhouse_config.db_name,
                clickhouse_config.host,
                clickhouse_config.host_port.to_string(),
                clickhouse_config.user,
                clickhouse_config.password,
                clickhouse_config.use_ssl.to_string(),
            ],
        };

        let child = run_python_program(program).unwrap();
        let output = child.wait_with_output().await.unwrap();

        //print output stdout and stderr
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }
}
