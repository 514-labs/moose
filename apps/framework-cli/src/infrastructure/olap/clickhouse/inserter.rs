use crate::infrastructure::olap::clickhouse::client::ClickHouseClient;
use crate::infrastructure::olap::clickhouse::model::ClickHouseRecord;
use std::time::Duration;

use log::{debug, error};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time;

use super::config::ClickHouseConfig;

const MAX_FLUSH_INTERVAL_SECONDS: u64 = 1;
const MAX_BATCH_SIZE: usize = 1000;

pub type BatchRecords = Arc<Mutex<Vec<ClickHouseRecord>>>;

pub struct Inserter {
    buffer: BatchRecords,
}

// TODO Add at least once delivery guarantees
impl Inserter {
    pub fn new(clickhouse_config: ClickHouseConfig, table: String, columns: Vec<String>) -> Self {
        let buffer = Arc::new(Mutex::new(Vec::new()));

        tokio::spawn(flush(clickhouse_config, buffer.clone(), table, columns));

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
                    error!("Error inserting records: {:?}", e);
                }
            }
        }

        buffer_owned.clear();

        drop(buffer_owned);
    }
}
