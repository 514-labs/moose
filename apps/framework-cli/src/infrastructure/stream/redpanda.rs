use log::info;
use rdkafka::{
    producer::{FutureProducer, Producer},
    ClientConfig,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{infrastructure::stream::rpk, utilities::docker};

// TODO: We need to configure the application based on the current project directory structure to ensure that we catch changes made outside of development mode

// Creates a topic from a file name
pub fn create_topic_from_name(project_name: &str, topic_name: String) -> std::io::Result<String> {
    info!("Creating topic: {}", topic_name);
    docker::run_rpk_command(
        project_name,
        rpk::create_rpk_command_args(rpk::RPKCommand::Topic(rpk::TopicCommand::Create {
            topic_name,
        })),
    )
}

// Deletes a topic from a file name
pub fn delete_topic(project_name: &str, topic_name: &str) -> std::io::Result<String> {
    info!("Deleting topic: {}", topic_name);
    let valid_topic_name = topic_name.to_lowercase();
    docker::run_rpk_command(
        project_name,
        rpk::create_rpk_command_args(rpk::RPKCommand::Topic(rpk::TopicCommand::Delete {
            topic_name: valid_topic_name,
        })),
    )
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedpandaConfig {
    pub broker: String,
    pub message_timeout_ms: i32,
    pub sasl_username: Option<String>,
    pub sasl_password: Option<String>,
    pub sasl_mechanism: Option<String>,
    pub security_protocol: Option<String>,
}

impl Default for RedpandaConfig {
    fn default() -> Self {
        Self {
            broker: "localhost:19092".to_string(),
            message_timeout_ms: 1000,
            sasl_username: None,
            sasl_password: None,
            sasl_mechanism: None,
            security_protocol: None,
        }
    }
}

#[derive(Clone)]
pub struct ConfiguredProducer {
    pub producer: FutureProducer,
    pub config: RedpandaConfig,
}

fn config_client(config: &RedpandaConfig) -> ClientConfig {
    let mut client_config = ClientConfig::new();
    client_config
        .set("bootstrap.servers", config.clone().broker)
        .set(
            "message.timeout.ms",
            config.clone().message_timeout_ms.to_string(),
        );

    if let Some(username) = config.clone().sasl_username {
        client_config.set("sasl.username", &username);
    }
    if let Some(password) = config.clone().sasl_password {
        client_config.set("sasl.password", &password);
    }
    if let Some(mechanism) = config.clone().sasl_mechanism {
        client_config.set("sasl.mechanism", &mechanism);
    }
    if let Some(security_protocol) = config.clone().security_protocol {
        client_config.set("security.protocol", &security_protocol);
    }
    client_config
}

pub fn create_producer(config: RedpandaConfig) -> ConfiguredProducer {
    let client_config = config_client(&config);
    let producer = client_config.create().expect("Failed to create producer");
    ConfiguredProducer { producer, config }
}

pub async fn fetch_topics(
    config: &RedpandaConfig,
) -> Result<Vec<String>, rdkafka::error::KafkaError> {
    let client_config = config_client(&config);
    let producer: FutureProducer = client_config.create().expect("Failed to create producer");
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
