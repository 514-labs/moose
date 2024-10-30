use crate::infrastructure::olap::clickhouse::client::ClickHouseClientTrait;
use crate::infrastructure::olap::clickhouse::model::ClickHouseRecord;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use log::{debug, error};
use rdkafka::error::KafkaError;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time;

const MAX_FLUSH_INTERVAL_SECONDS: u64 = 1;
const MAX_BATCH_SIZE: usize = 100000;

#[derive(Default)]
pub struct Batch {
    pub records: Vec<ClickHouseRecord>,
    pub partition_offsets: HashMap<i32, i64>,
}

impl Batch {
    fn update_offset(&mut self, partition: i32, offset: i64) {
        self.partition_offsets
            .entry(partition)
            .and_modify(|e| *e = (*e).max(offset))
            .or_insert(offset);
    }
}

pub type OffsetCommitCallback = Box<dyn Fn(i32, i64) -> Result<(), KafkaError> + Send + Sync>;
pub type BatchQueue = Arc<Mutex<VecDeque<Batch>>>;

pub struct Inserter<C: ClickHouseClientTrait + 'static> {
    queue: BatchQueue,
    client: Arc<C>,
}

impl<C: ClickHouseClientTrait + 'static> Inserter<C> {
    pub fn new(
        client: Arc<C>,
        table: &str,
        columns: Vec<String>,
        commit_callback: OffsetCommitCallback,
    ) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::from([Batch::default()])));

        let inserter = Self {
            queue: queue.clone(),
            client,
        };

        let inserter_clone = inserter.clone_for_flush();
        let table_clone = table.to_string();
        tokio::spawn(async move {
            inserter_clone
                .flush(table_clone, columns, commit_callback)
                .await;
        });

        inserter
    }

    fn clone_for_flush(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            client: self.client.clone(),
        }
    }

    pub async fn insert(
        &self,
        record: ClickHouseRecord,
        partition: i32,
        offset: i64,
    ) -> anyhow::Result<()> {
        let mut queue = self.queue.lock().await;

        let current_batch = queue.back_mut().unwrap();

        if current_batch.records.len() >= MAX_BATCH_SIZE {
            queue.push_back(Batch::default());
            let new_batch = queue.back_mut().unwrap();
            new_batch.records.push(record);
            new_batch.update_offset(partition, offset);
        } else {
            current_batch.records.push(record);
            current_batch.update_offset(partition, offset);
        }

        Ok(())
    }

    async fn flush(
        &self,
        table: String,
        columns: Vec<String>,
        commit_callback: OffsetCommitCallback,
    ) {
        let mut interval = time::interval(Duration::from_secs(MAX_FLUSH_INTERVAL_SECONDS));

        loop {
            interval.tick().await;
            let mut queue = self.queue.lock().await;

            if queue.is_empty() || queue.front().map_or(true, |batch| batch.records.is_empty()) {
                continue;
            }

            if let Some(batch) = queue.front() {
                let batch_size = batch.records.len();

                match self.client.insert(&table, &columns, &batch.records).await {
                    Ok(_) => {
                        debug!("Inserted {} records", batch_size,);

                        for (partition, offset) in &batch.partition_offsets {
                            let _ = commit_callback(*partition, *offset).map_err(|err| {
                                error!(
                                    "Error committing offset for partition {}: {:?}",
                                    partition, err
                                );
                            });
                        }

                        queue.pop_front();

                        if queue.is_empty() {
                            queue.push_back(Batch::default());
                        }
                    }
                    Err(e) => {
                        error!("Error inserting records to {}: {:?}", table, e);
                        debug!("Failed batch size: {}", batch_size);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::infrastructure::olap::clickhouse::model::ClickHouseValue;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct MockClickHouseClient {
        insert_calls: Arc<AtomicUsize>,
        should_fail: bool,
    }

    impl MockClickHouseClient {
        fn new(should_fail: bool) -> Arc<Self> {
            Arc::new(Self {
                insert_calls: Arc::new(AtomicUsize::new(0)),
                should_fail,
            })
        }
    }

    #[async_trait]
    impl ClickHouseClientTrait for MockClickHouseClient {
        async fn insert(
            &self,
            _table: &str,
            _columns: &[String],
            _records: &[ClickHouseRecord],
        ) -> anyhow::Result<()> {
            self.insert_calls.fetch_add(1, Ordering::SeqCst);
            if self.should_fail {
                Err(anyhow::anyhow!("Mock insert error"))
            } else {
                Ok(())
            }
        }
    }

    fn create_test_record(value: i64) -> ClickHouseRecord {
        let mut record = ClickHouseRecord::new();
        record.insert("test".to_string(), ClickHouseValue::new_int_64(value));
        record
    }

    #[tokio::test]
    async fn test_batch_creation_and_size_limit() {
        let mock_client = MockClickHouseClient::new(false);
        let inserter = Inserter::new(
            mock_client,
            "test_table",
            vec!["test".to_string()],
            Box::new(|_, _| Ok(())),
        );

        // Insert MAX_BATCH_SIZE records
        for i in 0..MAX_BATCH_SIZE {
            inserter
                .insert(create_test_record(i as i64), 0, i as i64)
                .await
                .unwrap();
        }

        // Add one more record to trigger new batch creation
        inserter
            .insert(
                create_test_record(MAX_BATCH_SIZE as i64),
                0,
                MAX_BATCH_SIZE as i64,
            )
            .await
            .unwrap();

        let queue = inserter.queue.lock().await;
        assert_eq!(queue.len(), 2, "Should have created two batches");

        let first_batch = queue.front().unwrap();
        assert_eq!(
            first_batch.records.len(),
            MAX_BATCH_SIZE,
            "First batch should be full"
        );

        let second_batch = queue.back().unwrap();
        assert_eq!(
            second_batch.records.len(),
            1,
            "Second batch should have one record"
        );
    }

    #[tokio::test]
    async fn test_successful_flush() {
        let mock_client = MockClickHouseClient::new(false);
        let insert_calls = mock_client.insert_calls.clone();

        let inserter = Inserter::new(
            mock_client,
            "test_table",
            vec!["test".to_string()],
            Box::new(|_, _| Ok(())),
        );

        inserter
            .insert(create_test_record(1), 0, 100)
            .await
            .unwrap();
        inserter
            .insert(create_test_record(2), 1, 200)
            .await
            .unwrap();

        // Wait for flush interval
        tokio::time::sleep(Duration::from_secs(MAX_FLUSH_INTERVAL_SECONDS + 1)).await;

        assert_eq!(
            insert_calls.load(Ordering::SeqCst),
            1,
            "Mock client should have been called once"
        );

        let queue = inserter.queue.lock().await;
        assert_eq!(
            queue.front().unwrap().records.len(),
            0,
            "Batch should be empty after successful flush"
        );
    }

    #[tokio::test]
    async fn test_failed_flush() {
        let mock_client = MockClickHouseClient::new(true);
        let insert_calls = mock_client.insert_calls.clone();

        let inserter = Inserter::new(
            mock_client,
            "test_table",
            vec!["test".to_string()],
            Box::new(|_, _| Ok(())),
        );

        inserter
            .insert(create_test_record(1), 0, 100)
            .await
            .unwrap();

        // Wait for flush interval
        tokio::time::sleep(Duration::from_secs(MAX_FLUSH_INTERVAL_SECONDS + 1)).await;

        assert_eq!(
            insert_calls.load(Ordering::SeqCst),
            1,
            "Mock client should have been called once"
        );

        let queue = inserter.queue.lock().await;
        assert_eq!(
            queue.front().unwrap().records.len(),
            1,
            "Records should remain in batch after failed flush"
        );
    }

    #[tokio::test]
    async fn test_offset_tracking_per_partition() {
        let mock_client = MockClickHouseClient::new(false);
        let inserter = Inserter::new(
            mock_client,
            "test_table",
            vec!["test".to_string()],
            Box::new(|_, _| Ok(())),
        );

        inserter
            .insert(create_test_record(1), 0, 100)
            .await
            .unwrap();
        inserter
            .insert(create_test_record(2), 1, 200)
            .await
            .unwrap();
        inserter
            .insert(create_test_record(3), 0, 150)
            .await
            .unwrap();
        inserter
            .insert(create_test_record(4), 1, 175)
            .await
            .unwrap();

        let queue = inserter.queue.lock().await;
        let batch = queue.front().unwrap();

        assert_eq!(
            *batch.partition_offsets.get(&0).unwrap(),
            150,
            "Should track highest offset for partition 0"
        );
        assert_eq!(
            *batch.partition_offsets.get(&1).unwrap(),
            200,
            "Should track highest offset for partition 1"
        );
    }
}
