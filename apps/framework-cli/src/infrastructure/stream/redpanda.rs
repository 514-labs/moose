use log::{error, info};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    producer::{FutureProducer, Producer},
    ClientConfig,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// TODO: We need to configure the application based on the current project directory structure to ensure that we catch changes made outside of development mode

pub async fn create_topics(
    config: &RedpandaConfig,
    topics: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating topics: {:?}", topics);

    let admin_client: AdminClient<_> = config_client(config)
        .create()
        .expect("Redpanda Admin Client creation failed");

    // Prepare the AdminOptions
    let options = AdminOptions::new().operation_timeout(Some(std::time::Duration::from_secs(5)));

    for topic_name in &topics {
        // Create a new topic with 1 partition and replication factor 1
        let topic = NewTopic::new(topic_name, 1, TopicReplication::Fixed(1));

        // Set some topic configurations
        let topic = topic
            .set("retention.ms", "1000")
            .set("segment.bytes", "10000");

        let result_list = admin_client.create_topics(&[topic], &options).await?;

        for result in result_list {
            match result {
                Ok(topic_name) => info!("Topic {} created successfully", topic_name),
                Err((topic_name, err)) => {
                    error!("Failed to create topic {}: {}", topic_name, err)
                }
            }
        }
    }

    Ok(())
}

pub async fn delete_topics(
    config: &RedpandaConfig,
    topics: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Deleting topics: {:?}", topics);

    let admin_client: AdminClient<_> = config_client(config)
        .create()
        .expect("Redpanda Admin Client creation failed");

    // Prepare the AdminOptions
    let options = AdminOptions::new().operation_timeout(Some(std::time::Duration::from_secs(5)));

    for topic_name in &topics {
        let result_list = admin_client
            .delete_topics(&[topic_name.as_str()], &options)
            .await?;

        for result in result_list {
            match result {
                Ok(topic_name) => info!("Topic {} deleted successfully", topic_name),
                Err((topic_name, err)) => {
                    error!("Failed to delete topic {}: {}", topic_name, err)
                }
            }
        }
    }

    Ok(())
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
    let client_config = config_client(config);
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

pub fn create_subscriber(config: &RedpandaConfig, group_id: &str, topic: &str) -> StreamConsumer {
    let mut client_config = config_client(config);

    client_config
        .set("session.timeout.ms", "6000")
        .set("enable.partition.eof", "false")
        .set("group.id", group_id);

    let consumer: StreamConsumer = client_config.create().expect("Failed to create consumer");

    let topics = [topic];

    consumer
        .subscribe(&topics)
        .expect("Can't subscribe to specified topic");

    consumer
}
