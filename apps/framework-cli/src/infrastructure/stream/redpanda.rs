use crate::framework::core::infrastructure::topic::Topic;
use crate::framework::core::infrastructure_map::{Change, StreamingChange};
use crate::project::Project;
use log::{error, info, warn};
use rdkafka::admin::{AlterConfig, NewPartitions, ResourceSpecifier};
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
    #[error("Not Supported - {0}")]
    NotSupported(String),

    #[error("Anyhow Error")]
    Other(#[from] anyhow::Error),
}

/// Validates changes to streaming configurations, particularly focused on topic partition counts
///
/// # Arguments
/// * `changes` - A slice of streaming changes to validate
///
/// # Returns
/// * `Ok(())` if all changes are valid
/// * `Err(RedpandaChangesError)` if any changes are invalid
///
/// # Errors
/// * Returns error if attempting to create a topic with 0 partitions
/// * Returns error if attempting to decrease partition count on an existing topic
pub fn validate_changes(changes: &[StreamingChange]) -> Result<(), RedpandaChangesError> {
    for change in changes.iter() {
        match change {
            StreamingChange::Topic(Change::Added(topic)) => {
                if topic.partition_count == 0 {
                    return Err(RedpandaChangesError::NotSupported(
                        "Partition count cannot be 0".to_string(),
                    ));
                }
            }

            StreamingChange::Topic(Change::Updated { before, after }) => {
                if before.partition_count > after.partition_count {
                    return Err(RedpandaChangesError::NotSupported(format!(
                        "Cannot decrease parallelism from {:?} to {:?}",
                        before.partition_count, after.partition_count
                    )));
                }
            }
            _ => {}
        }
    }

    Ok(())
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
                create_topics(&project.redpanda_config, vec![topic]).await?;
            }

            StreamingChange::Topic(Change::Removed(topic)) => {
                info!("Deleting topic: {:?}", topic.id());
                delete_topics(&project.redpanda_config, vec![topic.id()]).await?;
            }

            StreamingChange::Topic(Change::Updated { before, after }) => {
                if before.retention_period != after.retention_period {
                    info!("Updating topic: {:?} with: {:?}", before, after);
                    update_topic_config(&project.redpanda_config, &before.id(), after).await?;
                }

                if before.partition_count > after.partition_count {
                    warn!(
                        "{:?} Cannot decrease a partion size, ignoring the change",
                        before
                    );
                } else if before.partition_count < after.partition_count {
                    info!(
                        "Setting partitions count for topic: {:?} with: {:?}",
                        before.id(),
                        after.partition_count
                    );
                    add_partitions(
                        &project.redpanda_config,
                        &before.id(),
                        after.partition_count,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

/// Adds new partitions to an existing Redpanda/Kafka topic
///
/// # Arguments
/// * `redpanda_config` - Configuration for connecting to Redpanda/Kafka cluster
/// * `id` - Name/ID of the topic to add partitions to
/// * `partition_count` - The total number of partitions the topic should have after the operation
///
/// # Returns
/// * `Ok(())` if partitions were added successfully
/// * `Err(anyhow::Error)` if operation failed
///
/// # Errors
/// * Returns error if admin client creation fails
/// * Returns error if partition creation request fails
/// * Returns error if operation times out (after 5 seconds)
async fn add_partitions(
    redpanda_config: &RedpandaConfig,
    id: &str,
    partition_count: usize,
) -> anyhow::Result<()> {
    info!("Adding partitions to topic: {}", id);

    let admin_client: AdminClient<_> = config_client(redpanda_config)
        .create()
        .expect("Redpanda Admin Client creation failed");

    let options = AdminOptions::new().operation_timeout(Some(Duration::from_secs(5)));
    let new_partitions = NewPartitions::new(id, partition_count);
    let result = admin_client
        .create_partitions([&new_partitions], &options)
        .await?;

    for res in result {
        match res {
            Ok(_) => info!("Topic {} set with {} partitions", id, partition_count),
            Err((_, err)) => {
                error!("Failed to add partitions to topic {}: {}", id, err);
                return Err(err.into());
            }
        }
    }

    Ok(())
}

async fn update_topic_config(
    redpanda_config: &RedpandaConfig,
    id: &str,
    after: &crate::framework::core::infrastructure::topic::Topic,
) -> anyhow::Result<()> {
    info!("Updating topic config for: {}", id);

    let admin_client: AdminClient<_> = config_client(redpanda_config)
        .create()
        .expect("Redpanda Admin Client creation failed");

    let options = AdminOptions::new().operation_timeout(Some(Duration::from_secs(5)));

    let retention_ms = after.retention_period.as_millis().to_string();

    let topic = ResourceSpecifier::Topic(id);
    let config = AlterConfig::new(topic).set("retention.ms", retention_ms.as_str());

    let result = admin_client.alter_configs(&[config], &options).await?;

    for res in result {
        match res {
            Ok(_) => info!("Topic {} config updated successfully", id),
            Err((_, err)) => {
                error!("Failed to update config for topic {}: {}", id, err);
                return Err(err.into());
            }
        }
    }

    Ok(())
}

// TODO: We need to configure the application based on the current project directory structure to
// ensure that we catch changes made outside of development mode

// TODO: We need to make this a proper client so that we don't have
// to reinstantiate the client every time we want to use it

pub async fn create_topics(config: &RedpandaConfig, topics: Vec<&Topic>) -> anyhow::Result<()> {
    info!("Creating topics: {:?}", topics);

    let admin_client: AdminClient<_> = config_client(config)
        .create()
        .expect("Redpanda Admin Client creation failed");

    // Prepare the AdminOptions
    let options = AdminOptions::new().operation_timeout(Some(std::time::Duration::from_secs(5)));

    for topic in &topics {
        // Create a new topic with 1 partition and replication factor 1
        let new_topic = NewTopic::new(
            &topic.name,
            topic.partition_count as i32,
            TopicReplication::Fixed(config.replication_factor),
        );

        let topic_retention = topic.retention_period.as_millis().to_string();

        // Set some topic configurations
        let new_topic = new_topic
            .set("retention.ms", &topic_retention)
            .set("segment.bytes", "10000");

        let result_list = admin_client.create_topics(&[new_topic], &options).await?;

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

    pub fn prefix_with_namespace(&self, value: &str) -> String {
        format!("{}{}", self.get_namespace_prefix(), value)
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
        .set("acks", "all")
        .set("enable.gapless.guarantee", true.to_string());
    client_config.create().expect("Failed to create producer")
}

pub fn create_producer(config: RedpandaConfig) -> ConfiguredProducer {
    let mut client_config = config_client(&config);

    client_config.set(
        "message.timeout.ms",
        config.clone().message_timeout_ms.to_string(),
    );
    // This means that all the In Sync replicas need to acknowledge the message
    // before the message is considered sent.
    // Idempotence implies acks = all but idempotence is a stronger garantee (exactly once)
    // We currently only want at least once delivery
    client_config.set("acks", "all");
    // This is the maximum number of retries that will be made before the timeout.
    client_config.set("retries", "2147483647");

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
        .filter(|t| t.name().starts_with(&config.get_namespace_prefix()))
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
        .set("enable.auto.offset.store", "false")
        // to read records sent before subscriber is created
        .set("auto.offset.reset", "earliest")
        // Groupid
        .set("group.id", config.prefix_with_namespace(group_id))
        .set("isolation.level", "read_committed");

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framework::versions::Version;

    #[test]
    fn test_validate_changes_zero_partitions() {
        let topic = Topic {
            version: Version::from_string("1.0.0".to_string()),
            name: "test_topic".to_string(),
            retention_period: Duration::from_secs(60),
            partition_count: 0,
            columns: vec![],
            source_primitive: crate::framework::core::infrastructure_map::PrimitiveSignature {
                name: "test".to_string(),
                primitive_type:
                    crate::framework::core::infrastructure_map::PrimitiveTypes::DataModel,
            },
        };

        let changes = vec![StreamingChange::Topic(Change::Added(Box::new(topic)))];

        assert!(validate_changes(&changes).is_err());
    }

    #[test]
    fn test_validate_changes_decrease_partitions() {
        let before = Topic {
            version: Version::from_string("1.0.0".to_string()),
            name: "test_topic".to_string(),
            retention_period: Duration::from_secs(60),
            partition_count: 3,
            columns: vec![],
            source_primitive: crate::framework::core::infrastructure_map::PrimitiveSignature {
                name: "test".to_string(),
                primitive_type:
                    crate::framework::core::infrastructure_map::PrimitiveTypes::DataModel,
            },
        };

        let after = Topic {
            partition_count: 1,
            ..before.clone()
        };

        let changes = vec![StreamingChange::Topic(Change::Updated {
            before: Box::new(before),
            after: Box::new(after),
        })];

        assert!(validate_changes(&changes).is_err());
    }

    #[test]
    fn test_validate_changes_valid() {
        let before = Topic {
            version: Version::from_string("1.0.0".to_string()),
            name: "test_topic".to_string(),
            retention_period: Duration::from_secs(60),
            partition_count: 1,
            columns: vec![],
            source_primitive: crate::framework::core::infrastructure_map::PrimitiveSignature {
                name: "test".to_string(),
                primitive_type:
                    crate::framework::core::infrastructure_map::PrimitiveTypes::DataModel,
            },
        };

        let after = Topic {
            partition_count: 3,
            ..before.clone()
        };

        let changes = vec![StreamingChange::Topic(Change::Updated {
            before: Box::new(before),
            after: Box::new(after),
        })];

        assert!(validate_changes(&changes).is_ok());
    }
}
