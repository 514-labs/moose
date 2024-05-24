use std::collections::HashMap;

use std::process::Child;

use crate::infrastructure::stream::redpanda::RedpandaConfig;

use super::model::{flow_id, Flow, FlowError};

/**
 * The process registry manages the lifecycle of the flows.
 */
pub struct FlowProcessRegistry {
    registry: HashMap<String, Child>,
    kafka_config: RedpandaConfig,
}

impl FlowProcessRegistry {
    pub fn new(kafka_config: RedpandaConfig) -> Self {
        Self {
            registry: HashMap::new(),
            kafka_config,
        }
    }

    pub fn start(&mut self, flow: Flow) -> Result<(), FlowError> {
        let child = flow.start(self.kafka_config.clone())?;

        self.registry.insert(flow.id(), child);

        Ok(())
    }

    pub fn start_all(&mut self, flows: &[Flow]) -> Result<(), FlowError> {
        for flow in flows {
            self.start(flow.clone())?;
        }

        Ok(())
    }

    pub async fn stop(&mut self, source_topic: &str, target_topic: &str) -> Result<(), FlowError> {
        let flow_id = flow_id(source_topic, target_topic);

        if let Some(running_flow) = self.registry.get_mut(&flow_id) {
            running_flow.kill()?;
            self.registry.remove(&flow_id);
        }

        Ok(())
    }

    pub async fn stop_all(&mut self) -> Result<(), FlowError> {
        for (_, running_flow) in self.registry.iter_mut() {
            running_flow.kill()?;
        }

        self.registry.clear();

        Ok(())
    }
}
