use crate::infrastructure::olap::clickhouse::client::ClickHouseClient;
use crate::infrastructure::olap::clickhouse::config::ClickHouseConfig;
use crate::infrastructure::stream::redpanda::create_subscriber;
use crate::infrastructure::stream::redpanda::RedpandaConfig;
use futures::{StreamExt, TryStreamExt};
use rdkafka::message::{BorrowedMessage, OwnedMessage};
use rdkafka::Message;

use log::debug;

const SYNC_GROUP_ID: &str = "clickhouse_sync";

// TODO - add the sync start to dev mode and prod mode

pub async fn sync_kafka_to_clickhouse(
    kafka_config: &RedpandaConfig,
    clickhouse_config: &ClickHouseConfig,
    streaming_topic: &str,
    clickhouse_table: &str,
) -> anyhow::Result<()> {
    let subscriber = create_subscriber(kafka_config, SYNC_GROUP_ID, streaming_topic);
    let clickhouse_client = ClickHouseClient::new(clickhouse_config).await?;

    subscriber
        .stream()
        .try_for_each(|message| {
            async move {
                let owned_message = message.detach();
                match owned_message.payload() {
                    Some(payload) => {
                        let payload_str = std::str::from_utf8(payload).unwrap();

                        debug!("Received message: {}", payload_str);
                        println!("Received message: {}", payload_str);

                        // clickhouse_client
                        //     .insert(clickhouse_table, payload_str)
                        //     .await?;
                    }
                    None => {
                        debug!("Received message with no payload");
                        println!("Received message with no payload");
                    }
                }

                // Insert into clickhouse
                Ok(())
            }
        })
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_sync() {
    let kafka_config = RedpandaConfig {
        broker: "localhost:19092".to_string(),
        sasl_username: None,
        sasl_password: None,
        sasl_mechanism: None,
        security_protocol: None,
        message_timeout_ms: 5000,
    };

    let clickhouse_config = ClickHouseConfig {
        user: "panda".to_string(),
        password: "pandapass".to_string(),
        host: "localhost".to_string(),
        use_ssl: false,
        postgres_port: 5432,
        kafka_port: 9092,
        host_port: 18123,
        db_name: "local".to_string(),
    };

    let streaming_topic = "UserActivity_0_0";
    let clickhouse_table = "UserActivity_0_0";

    sync_kafka_to_clickhouse(
        &kafka_config,
        &clickhouse_config,
        streaming_topic,
        clickhouse_table,
    )
    .await
    .unwrap();
}
