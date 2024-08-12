use crate::framework::core::infrastructure_map::{Change, StreamingChange};
use crate::project::Project;
use log::{error, info, warn};
use rdkafka::admin::ResourceSpecifier;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::Consumer;
use rdkafka::error::KafkaError;
use rdkafka::producer::{DeliveryFuture, FutureRecord};
use rdkafka::types::RDKafkaErrorCode;
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic, TopicReplication},
    producer::{FutureProducer, Producer},
    ClientConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum RedpandaChangesError {
    #[error("Not Supported {0}")]
    NotSupported(String),

    #[error("Anyhow Error")]
    Other(#[from] anyhow::Error),
}

pub async fn execute_changes(
    project: &Project,
    changes: &[StreamingChange],
) -> Result<(), RedpandaChangesError> {
    // TODO: we need to take into account all the current state of the topic,
    // ie the retention period and bytes sizes. But we will need to change the
    // interfaces of create method to do that properly. We will do it once we have
    // move to the new core.

    for change in changes.iter() {
        match change {
            StreamingChange::Topic(Change::Added(topic)) => {
                info!("Creating topic: {:?}", topic.id());
                create_topics(&project.redpanda_config, vec![topic.id()]).await?;
            }

            StreamingChange::Topic(Change::Removed(topic)) => {
                info!("Deleting topic: {:?}", topic.id());
                delete_topics(&project.redpanda_config, vec![topic.id()]).await?;
            }

            StreamingChange::Topic(Change::Updated { before, after }) => {
                if !project.is_production {
                    info!("Replacing topic: {:?} with: {:?}", before, after);
                    delete_topics(&project.redpanda_config, vec![before.id()]).await?;
                    create_topics(&project.redpanda_config, vec![after.id()]).await?;
                } else {
                    return Err(RedpandaChangesError::NotSupported(format!(
                        "Updating topic {} is not supported in production mode",
                        before.id()
                    )));
                }
            }
        }
    }

    Ok(())
}

// TODO: We need to configure the application based on the current project directory structure to
// ensure that we catch changes made outside of development mode

// TODO: We need to make this a proper client so that we don't have
// to reinstantiate the client every time we want to use it

pub async fn create_topics(config: &RedpandaConfig, topics: Vec<String>) -> anyhow::Result<()> {
    info!("Creating topics: {:?}", topics);

    let admin_client: AdminClient<_> = config_client(config)
        .create()
        .expect("Redpanda Admin Client creation failed");

    // Prepare the AdminOptions
    let options = AdminOptions::new().operation_timeout(Some(std::time::Duration::from_secs(5)));

    let retention_ms = config.retention_ms.to_string();

    for topic_name in &topics {
        // Create a new topic with 1 partition and replication factor 1
        let topic = NewTopic::new(
            topic_name,
            1,
            TopicReplication::Fixed(config.replication_factor),
        );

        // Set some topic configurations
        let topic = topic
            .set("retention.ms", retention_ms.as_str())
            .set("segment.bytes", "10000");

        let result_list = admin_client.create_topics(&[topic], &options).await?;

        for result in result_list {
            match result {
                Ok(topic_name) => info!("Topic {} created successfully", topic_name),
                Err((topic_name, RDKafkaErrorCode::TopicAlreadyExists)) => {
                    info!("Topic {} already exists", topic_name)
                }
                Err((topic_name, err)) => {
                    error!("Failed to create topic {}: {}", topic_name, err);
                    return Err(err.into());
                }
            }
        }
    }

    Ok(())
}

pub async fn delete_topics(
    config: &RedpandaConfig,
    topics: Vec<String>,
) -> Result<(), anyhow::Error> {
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

pub async fn describe_topic_config(
    config: &RedpandaConfig,
    topic_name: &str,
) -> Result<HashMap<String, String>, rdkafka::error::KafkaError> {
    info!("Describing config for topic: {}", topic_name);

    let admin_client: AdminClient<_> = config_client(config)
        .create()
        .expect("Redpanda Admin Client creation failed");

    let options = AdminOptions::new().operation_timeout(Some(std::time::Duration::from_secs(5)));

    let result = admin_client
        .describe_configs(&[ResourceSpecifier::Topic(topic_name)], &options)
        .await?;

    match result.into_iter().next() {
        Some(Ok(config_resource)) => {
            let config_map: HashMap<String, String> = config_resource
                .entry_map()
                .into_iter()
                .filter_map(|(config_name, config_entry)| {
                    config_entry
                        .value
                        .as_ref()
                        .map(|value| (config_name.to_string(), value.to_string()))
                })
                .collect();

            Ok(config_map)
        }
        Some(Err(err)) => {
            error!("Failed to describe topic config: {}", err);
            Ok(HashMap::new())
        }
        None => {
            error!("No response from describe_configs");
            Ok(HashMap::new())
        }
    }
}

const NAMESPACE_SEPARATOR: &str = ".";

fn default_replication_factor() -> i32 {
    1
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RedpandaConfig {
    pub broker: String,
    pub message_timeout_ms: i32,
    pub retention_ms: i32,
    #[serde(default = "default_replication_factor")]
    pub replication_factor: i32,
    pub sasl_username: Option<String>,
    pub sasl_password: Option<String>,
    pub sasl_mechanism: Option<String>,
    pub security_protocol: Option<String>,
    pub namespace: Option<String>,
}

impl RedpandaConfig {
    pub fn get_namespace_prefix(&self) -> String {
        self.namespace
            .as_ref()
            .map(|ns| format!("{}{}", ns, NAMESPACE_SEPARATOR))
            .unwrap_or_default()
    }

    pub fn get_topic_with_namespace(&self, topic: &String) -> String {
        format!("{}{}", self.get_namespace_prefix(), topic)
    }

    pub fn get_topic_without_namespace(topic: &str) -> String {
        if let Some(pos) = topic.find(NAMESPACE_SEPARATOR) {
            topic[pos + 1..].to_string()
        } else {
            topic.to_owned()
        }
    }
}

impl Default for RedpandaConfig {
    fn default() -> Self {
        Self {
            broker: "localhost:19092".to_string(),
            message_timeout_ms: 1000,
            retention_ms: 30000,
            replication_factor: 1,
            sasl_username: None,
            sasl_password: None,
            sasl_mechanism: None,
            security_protocol: None,
            namespace: None,
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

    // to prevent the wrapped library from writing to stderr
    client_config.log_level = RDKafkaLogLevel::Emerg;

    client_config.set("bootstrap.servers", config.clone().broker);

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

pub fn create_idempotent_producer(config: &RedpandaConfig) -> FutureProducer {
    let mut client_config = config_client(config);

    client_config
        .set("message.timeout.ms", (5 * 60 * 1000).to_string())
        .set("enable.idempotence", true.to_string())
        .set("enable.gapless.guarantee", true.to_string());
    client_config.create().expect("Failed to create producer")
}

pub fn create_producer(config: RedpandaConfig) -> ConfiguredProducer {
    let mut client_config = config_client(&config);

    client_config.set(
        "message.timeout.ms",
        config.clone().message_timeout_ms.to_string(),
    );
    let producer = client_config.create().expect("Failed to create producer");
    ConfiguredProducer { producer, config }
}

pub async fn check_topic_size(topic: &str, config: &RedpandaConfig) -> Result<i64, KafkaError> {
    let client: StreamConsumer<_> = config_client(config).create()?;
    let timeout = Duration::from_secs(1);
    let md = client.fetch_metadata(Some(topic), timeout)?;
    let partitions = match md.topics().iter().find(|t| t.name() == topic) {
        None => return Ok(0),
        Some(topic) => topic.partitions(),
    };
    let total_count = partitions
        .iter()
        .map(|partition| {
            let (_, high_watermark) = client.fetch_watermarks(topic, partition.id(), timeout)?;
            Ok::<i64, KafkaError>(high_watermark)
        })
        .collect::<Result<Vec<i64>, _>>()?
        .into_iter()
        .sum();

    Ok(total_count)
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
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "1000")
        // to read records sent before subscriber is created
        .set("auto.offset.reset", "earliest")
        // Groupid
        .set("group.id", group_id);

    let consumer: StreamConsumer = client_config.create().expect("Failed to create consumer");

    let topics = [topic];

    consumer
        .subscribe(&topics)
        .expect("Can't subscribe to specified topic");

    consumer
}

pub async fn wait_for_delivery(topic: &str, future: DeliveryFuture) {
    match future.await {
        Ok(Ok((partition, offset))) => {
            if offset % 1024 == 0 {
                info!(
                    // the timestamp of this logging is not the same time Kafka receives the message
                    "Sent to {} partition {} offset {}",
                    topic, partition, offset
                )
            }
        }
        Ok(Err((error, _))) => {
            warn!(
                "Failed to deliver message to {} with error: {}",
                topic, error
            );
        }
        Err(cancelled) => {
            error!(
                "Kafka DeliveryFuture is {}. This should never happen.",
                cancelled
            );
        }
    }
}

async fn maybe_dequeue(topic: &str, queue: &mut VecDeque<DeliveryFuture>) {
    if queue.len() >= 2 << 16 {
        while queue.len() >= 2 << 15 {
            match queue.pop_front() {
                None => return,
                Some(f) => wait_for_delivery(topic, f).await,
            };
        }
    }
}
pub async fn send_with_back_pressure(
    queue: &mut VecDeque<DeliveryFuture>,
    producer: &FutureProducer,
    topic: &str,
    payload: String,
) {
    let mut record = FutureRecord::to(topic)
        .key(topic)
        .payload(payload.as_bytes());

    loop {
        let queue_result = producer.send_result(record);
        match queue_result {
            Ok(f) => {
                queue.push_back(f);
                maybe_dequeue(topic, queue).await;
                return;
            }
            Err((KafkaError::MessageProduction(RDKafkaErrorCode::QueueFull), r)) => {
                record = r;
                if let Some(f) = queue.pop_front() {
                    wait_for_delivery(topic, f).await;
                }
            }
            Err((unknown_err, _)) => {
                error!(
                    "Got unknown error {}. This should never happen.",
                    unknown_err
                );
                return;
            }
        }
    }
}
