use crate::infrastructure::olap::clickhouse::client::ClickHouseClientTrait;
use crate::infrastructure::olap::clickhouse::model::ClickHouseRecord;
use std::collections::{HashMap, VecDeque};

use log::{info, warn};
use rdkafka::error::KafkaError;

type Partition = i32;
type Offset = i64;
type PartitionOffsets = HashMap<Partition, Offset>;
type PartitionSizes = HashMap<Partition, i64>;

#[derive(Default)]
pub struct Batch {
    pub records: Vec<ClickHouseRecord>,
    pub partition_offsets: PartitionOffsets,

    // Map the partitions to the number of messages in the batch
    pub messages_sizes: PartitionSizes,
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
pub type BatchQueue = VecDeque<Batch>;

pub struct Inserter<C: ClickHouseClientTrait + 'static> {
    queue: BatchQueue,
    client: C,
    batch_size: usize,
    commit_callback: OffsetCommitCallback,
    table: String,
    columns: Vec<String>,
}

impl<C: ClickHouseClientTrait + 'static> Inserter<C> {
    pub fn new(
        client: C,
        batch_size: usize,
        commit_callback: OffsetCommitCallback,
        table: String,
        columns: Vec<String>,
    ) -> Self {
        let queue = VecDeque::from([Batch::default()]);

        Self {
            queue,
            client,
            batch_size,
            commit_callback,
            table,
            columns,
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn insert(&mut self, record: ClickHouseRecord, partition: i32, offset: i64) {
        let current_batch = self.queue.back_mut();

        match current_batch {
            Some(batch) => {
                if batch.records.len() >= self.batch_size {
                    self.queue.push_back(Batch::default());
                    let new_batch = self.queue.back_mut().unwrap();
                    new_batch.records.push(record);
                    new_batch.update_offset(partition, offset);
                } else {
                    batch.records.push(record);
                    batch.update_offset(partition, offset);
                }
            }
            None => {
                self.queue.push_back(Batch::default());
                let new_batch = self.queue.back_mut().unwrap();
                new_batch.records.push(record);
                new_batch.update_offset(partition, offset);
            }
        }
    }

    pub async fn flush(&mut self) {
        if self.queue.is_empty()
            || self
                .queue
                .front()
                .is_none_or(|batch| batch.records.is_empty())
        {
            return;
        }

        if let Some(batch) = self.queue.front() {
            match self
                .client
                .insert(&self.table, &self.columns, &batch.records)
                .await
            {
                Ok(_) => {
                    info!(
                        "Batch Insert records - table='{}';insert_sizes='{}';offsets='{}'",
                        self.table,
                        batch.messages_sizes_to_string(),
                        batch.offsets_to_string()
                    );
                    crate::cli::display::batch_inserted(batch.records.len(), &self.table);

                    for (partition, offset) in &batch.partition_offsets {
                        if let Err(err) = (self.commit_callback)(*partition, *offset) {
                            warn!(
                                "Error committing offset {} for partition {}: {:?}",
                                offset, partition, err
                            );
                        }
                    }

                    self.queue.pop_front();
                }
                Err(e) => {
                    warn!(
                        "Transient Failure - table='{}';insert_sizes='{}';offsets='{}';error='{}'",
                        self.table,
                        batch.messages_sizes_to_string(),
                        batch.offsets_to_string(),
                        e
                    );
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
    use std::sync::Arc;
    struct MockClickHouseClient {
        insert_calls: Arc<AtomicUsize>,
        should_fail: bool,
    }

    impl MockClickHouseClient {
        fn new(should_fail: bool) -> Self {
            Self {
                insert_calls: Arc::new(AtomicUsize::new(0)),
                should_fail,
            }
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
        let mut inserter = Inserter::new(
            mock_client,
            1,
            Box::new(|_, _| Ok(())),
            "test_table".to_string(),
            vec!["test".to_string()],
        );

        // Insert 2 records with batch size 1
        for i in 0..2 {
            inserter.insert(create_test_record(i as i64), 0, i as i64);
        }

        assert_eq!(inserter.queue.len(), 2, "Should have created two batches");

        let first_batch = inserter.queue.front().unwrap();
        assert_eq!(first_batch.records.len(), 1, "First batch should be full");

        let second_batch = inserter.queue.back().unwrap();
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

        let mut inserter = Inserter::new(
            mock_client,
            100,
            Box::new(|_, _| Ok(())),
            "test_table".to_string(),
            vec!["test".to_string()],
        );

        inserter.insert(create_test_record(1), 0, 100);
        inserter.insert(create_test_record(2), 1, 200);

        // Manually flush instead of waiting for interval
        inserter.flush().await;

        assert_eq!(
            insert_calls.load(Ordering::SeqCst),
            1,
            "Mock client should have been called once"
        );

        assert_eq!(
            inserter
                .queue
                .front()
                .unwrap_or(&Batch::default())
                .records
                .len(),
            0,
            "Batch should be empty after successful flush"
        );
    }

    #[tokio::test]
    async fn test_offset_tracking_per_partition() {
        let mock_client = MockClickHouseClient::new(false);
        let mut inserter = Inserter::new(
            mock_client,
            100,
            Box::new(|_, _| Ok(())),
            "test_table".to_string(),
            vec!["test".to_string()],
        );

        inserter.insert(create_test_record(1), 0, 100);
        inserter.insert(create_test_record(2), 1, 200);
        inserter.insert(create_test_record(3), 0, 150);
        inserter.insert(create_test_record(4), 1, 175);

        let batch = inserter.queue.front().unwrap();

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
