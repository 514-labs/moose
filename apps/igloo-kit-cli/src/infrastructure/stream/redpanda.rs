use log::debug;
use rdkafka::{
    producer::{FutureProducer, Producer},
    ClientConfig,
};
use serde::Deserialize;
use std::{
    io::{self, Write},
    time::Duration,
};

use crate::{infrastructure::stream::rpk, utilities::docker};

// TODO: We need to configure the application based on the current project directory structure to ensure that we catch changes made outside of development mode

// Creates a topic from a file name
pub fn create_topic_from_name(topic_name: String) {
    let output = docker::run_rpk_command(rpk::create_rpk_command_args(rpk::RPKCommand::Topic(
        rpk::TopicCommand::Create { topic_name },
    )));

    match output {
        Ok(o) => {
            io::stdout().write_all(&o.stdout).unwrap();
        }
        Err(err) => {
            debug!("Error: {}", err)
        }
    }
}

// Deletes a topic from a file name
pub fn delete_topic(topic_name: String) {
    let valid_topic_name = topic_name.to_lowercase();
    let output = docker::run_rpk_command(rpk::create_rpk_command_args(rpk::RPKCommand::Topic(
        rpk::TopicCommand::Delete {
            topic_name: valid_topic_name,
        },
    )));

    match output {
        Ok(o) => {
            io::stdout().write_all(&o.stdout).unwrap();
        }
        Err(err) => {
            debug!("Error: {}", err)
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RedpandaConfig {
    pub broker: String,
    pub message_timeout_ms: i32,
}

impl Default for RedpandaConfig {
    fn default() -> Self {
        Self {
            broker: "localhost:19092".to_string(),
            message_timeout_ms: 1000,
        }
    }
}

#[derive(Clone)]
pub struct ConfiguredProducer {
    pub producer: FutureProducer,
    pub config: RedpandaConfig,
}

pub fn create_producer(config: RedpandaConfig) -> ConfiguredProducer {
    let producer = ClientConfig::new()
        .set("bootstrap.servers", config.clone().broker)
        .set(
            "message.timeout.ms",
            config.clone().message_timeout_ms.to_string(),
        )
        .create()
        .expect("Failed to create producer");

    ConfiguredProducer { producer, config }
}

pub async fn fetch_topics(
    config: &RedpandaConfig,
) -> Result<Vec<String>, rdkafka::error::KafkaError> {
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.broker)
        .set("message.timeout.ms", config.message_timeout_ms.to_string())
        .create()
        .expect("Failed to create producer");

    let metadata = producer
        .client()
        .fetch_metadata(None, Duration::from_secs(5))?;
    let topics = metadata
        .topics()
        .iter()
        .map(|t| t.name().to_string())
        .collect();
    Ok(topics)
}
