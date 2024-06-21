use crate::infrastructure::olap::clickhouse::client::ClickHouseClient;
use crate::infrastructure::olap::clickhouse::model::ClickHouseRecord;
use std::collections::HashMap;
use std::time::Duration;

use log::{debug, error};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time;

use super::config::ClickHouseConfig;

const MAX_FLUSH_INTERVAL_SECONDS: u64 = 1;
const MAX_BATCH_SIZE: usize = 100000;

pub type BatchRecords = Arc<Mutex<Vec<ClickHouseRecord>>>;

pub struct Inserter {
    buffer: BatchRecords,
}

// TODO Add at least once delivery guarantees
impl Inserter {
    pub fn new(clickhouse_config: ClickHouseConfig, table: &str, columns: Vec<String>) -> Self {
        let buffer = Arc::new(Mutex::new(Vec::new()));

        tokio::spawn(flush(
            clickhouse_config,
            buffer.clone(),
            table.to_string(),
            columns,
        ));

        Self { buffer }
    }

    pub async fn insert(&self, record: ClickHouseRecord) -> anyhow::Result<()> {
        let mut buffer: tokio::sync::MutexGuard<'_, Vec<ClickHouseRecord>> =
            self.buffer.lock().await;
        buffer.push(record);
        Ok(())
    }
}

async fn flush(
    clickhouse_config: ClickHouseConfig,
    buffer: BatchRecords,
    table: String,
    columns: Vec<String>,
) {
    let mut interval = time::interval(Duration::from_secs(MAX_FLUSH_INTERVAL_SECONDS));

    let client = ClickHouseClient::new(&clickhouse_config).unwrap();
    //println!("CLIENT: {}", client);

    let mut i: usize = 0;
    //println!("COLUMNS: {}", columns.len());

    loop {
        interval.tick().await;
        let mut buffer_owned = buffer.lock().await;
        if buffer_owned.is_empty() {
            drop(buffer_owned);
            continue;
        }

        // in case the buffer is too big, we slice it and then we flush it
        for chunk in buffer_owned.chunks(MAX_BATCH_SIZE) {
            match client.insert(&table, &columns, chunk).await {
                Ok(_) => {
                    debug!("Inserted {} records", chunk.len());
                }
                Err(e) => {
                    //println!("HASHMAP: {}", serde_json::to_string(&data).unwrap())
                    println!("{}", e);
                    //println!("CHUNK: {}", chunk);
                    //println!("TEST: {}", chunk);
                    error!("Error inserting records to {}: {:?}", table, e);
                    debug!("Failed batch {:?}", chunk);
                }
            }
        }

        buffer_owned.clear();

        drop(buffer_owned);
    }
}
