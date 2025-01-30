use crate::infrastructure::olap::clickhouse::client::ClickHouseClientTrait;
use crate::infrastructure::olap::clickhouse::model::ClickHouseRecord;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use crate::infrastructure::processes::kafka_clickhouse_sync;
use futures::FutureExt;
use log::{info, warn};
use rdkafka::error::KafkaError;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time;

#[derive(Default)]
pub struct Batch {
    pub records: Vec<ClickHouseRecord>,
    pub partition_offsets: HashMap<i32, i64>,

    // Map the partitions to the number of messages in the batch
    pub messages_sizes: HashMap<i32, i64>,
}

impl Batch {
    fn update_offset(&mut self, partition: i32, offset: i64) {
        self.partition_offsets
            .entry(partition)
            .and_modify(|e| *e = (*e).max(offset))
            .or_insert(offset);

        self.messages_sizes
            .entry(partition)
            .and_modify(|e| *e += 1)
            .or_insert(1);
    }

    fn offsets_to_string(&self) -> String {
        self.partition_offsets
            .iter()
            .map(|(partition, offset)| format!("Partition {}: {}", partition, offset))
            .collect::<Vec<String>>()
            .join(", ")
    }

    fn messages_sizes_to_string(&self) -> String {
        self.messages_sizes
            .iter()
            .map(|(partition, size)| format!("Partition {}: {}", partition, size))
            .collect::<Vec<String>>()
            .join(", ")
    }
}

pub type OffsetCommitCallback = Box<dyn Fn(i32, i64) -> Result<(), KafkaError> + Send + Sync>;
pub type BatchQueue = Arc<Mutex<VecDeque<Batch>>>;

pub struct Inserter<C: ClickHouseClientTrait + 'static> {
    queue: BatchQueue,
    client: Arc<C>,
    batch_size: usize,
    flush_interval: u64,
}

impl<C: ClickHouseClientTrait + 'static> Inserter<C> {
    pub fn new(
        client: Arc<C>,
        batch_size: usize,
        flush_interval: u64,
        table: &str,
        columns: Vec<String>,
        commit_callback: OffsetCommitCallback,
    ) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::from([Batch::default()])));

        let inserter = Self {
            queue: queue.clone(),
            client,
            batch_size,
            flush_interval,
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
            batch_size: self.batch_size,
            flush_interval: self.flush_interval,
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

        if current_batch.records.len() >= self.batch_size {
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

    async fn get_front_batch(&self) -> Option<Batch> {
        let mut queue = self.queue.lock().await;

        if queue.is_empty() || queue.front().is_none_or(|batch| batch.records.is_empty()) {
            None
        } else {
            let batch = queue.pop_front();

            if queue.is_empty() {
                queue.push_back(Batch::default())
            }
            batch
        }
    }

    async fn return_batch_to_queue(&self, mut batch: Batch) {
        let mut queue = self.queue.lock().await;
        if queue
            .front()
            .is_some_and(|new| new.records.len() + batch.records.len() <= self.batch_size)
        {
            let mut new_batch = queue.front_mut().unwrap();

            // prepending batch.records to the new batch
            batch.records.append(&mut new_batch.records);
            std::mem::swap(&mut batch.records, &mut new_batch.records);

            batch
                .partition_offsets
                .into_iter()
                .for_each(|(partition, offset)| {
                    // no need to update if non-empty as the new batch must be newer than the old batch
                    new_batch
                        .partition_offsets
                        .entry(partition)
                        .or_insert(offset);
                });

            batch
                .messages_sizes
                .into_iter()
                .for_each(|(partition, size)| {
                    new_batch
                        .partition_offsets
                        .entry(partition)
                        .and_modify(|e| *e += size)
                        .or_insert(size);
                })
        } else {
            queue.push_front(batch);
        }
    }

    async fn flush(
        &self,
        table: String,
        columns: Vec<String>,
        commit_callback: OffsetCommitCallback,
    ) {
        let mut interval = time::interval(Duration::from_secs(self.flush_interval));
        let mut pause_receiver = kafka_clickhouse_sync::clickhouse_writing_pause_listener();

        loop {
            if *pause_receiver.borrow() {
                pause_receiver.changed().await.unwrap();
                continue;
            }
            interval.tick().await;

            if let Some(batch) = self.get_front_batch() {
                match self.client.insert(&table, &columns, &batch.records).await {
                    Ok(_) => {
                        info!(
                            "Batch Insert records - table='{}';insert_sizes='{}';offsets='{}'",
                            table,
                            batch.messages_sizes_to_string(),
                            batch.offsets_to_string()
                        );

                        for (partition, offset) in &batch.partition_offsets {
                            if let Err(err) = commit_callback(*partition, *offset) {
                                warn!(
                                    "Error committing offset {} for partition {}: {:?}",
                                    offset, partition, err
                                );
                            }
                        }
                    }
                    Err(e) => {
                        self.return_batch_to_queue(batch).await;
                        warn!(
                            "Transient Failure - table='{}';insert_sizes='{}';offsets='{}';error='{}'",
                            table,
                            batch.messages_sizes_to_string(),
                            batch.offsets_to_string(),
                            e
                        );
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
            1,
            100000,
            "test_table",
            vec!["test".to_string()],
            Box::new(|_, _| Ok(())),
        );

        // Insert MAX_BATCH_SIZE records
        for i in 0..2 {
            inserter
                .insert(create_test_record(i as i64), 0, i as i64)
                .await
                .unwrap();
        }

        let queue = inserter.queue.lock().await;
        assert_eq!(queue.len(), 2, "Should have created two batches");

        let first_batch = queue.front().unwrap();
        assert_eq!(first_batch.records.len(), 1, "First batch should be full");

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
            100,
            1,
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
        tokio::time::sleep(Duration::from_secs(1 + 1)).await;

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
    async fn test_offset_tracking_per_partition() {
        let mock_client = MockClickHouseClient::new(false);
        let inserter = Inserter::new(
            mock_client,
            100,
            100000,
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
