use std::collections::HashMap;

use tokio::process::Child;

use crate::infrastructure::stream::redpanda::RedpandaConfig;

use super::model::{flow_id, Flow, FlowError};

struct RunningFlow {
    flow: Flow,
    process: Child,
}

struct FlowProcessRegistry {
    registry: HashMap<String, RunningFlow>,
    kafka_config: RedpandaConfig,
}

impl FlowProcessRegistry {
    fn new(kafka_config: RedpandaConfig) -> Self {
        Self {
            registry: HashMap::new(),
            kafka_config,
        }
    }

    fn start(&mut self, flow: Flow) -> Result<(), FlowError> {
        let child = flow.start(self.kafka_config.clone())?;

        self.registry.insert(
            flow.id(),
            RunningFlow {
                flow,
                process: child,
            },
        );

        Ok(())
    }

    fn start_all(&mut self, flows: &[Flow]) -> Result<(), FlowError> {
        for flow in flows {
            self.start(flow.clone())?;
        }

        Ok(())
    }

    async fn stop(&mut self, source_topic: &str, target_topic: &str) -> Result<(), FlowError> {
        let flow_id = flow_id(source_topic, target_topic);

        if let Some(running_flow) = self.registry.get_mut(&flow_id) {
            running_flow.process.kill().await?;
            self.registry.remove(&flow_id);
        }

        Ok(())
    }
}
