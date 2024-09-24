use crate::metrics::MetricEvent;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time;

const MAX_FLUSH_INTERVAL_SECONDS: u64 = 10;
const MAX_BATCH_SIZE: usize = 1000;

pub type BatchEvents = Arc<Mutex<Vec<MetricEvent>>>;

#[derive(Clone)]
pub struct MetricsInserter {
    buffer: BatchEvents,
}

impl MetricsInserter {
    pub fn new() -> Self {
        let buffer = Arc::new(Mutex::new(Vec::new()));

        tokio::spawn(flush(buffer.clone()));

        Self { buffer }
    }

    pub async fn insert(&self, event: MetricEvent) -> anyhow::Result<()> {
        let mut buffer = self.buffer.lock().await;
        buffer.push(event);
        Ok(())
    }
}

async fn flush(buffer: BatchEvents) {
    let mut interval = time::interval(Duration::from_secs(MAX_FLUSH_INTERVAL_SECONDS));
    let client = Client::new();

    loop {
        println!("Flushing metrics");
        interval.tick().await;
        let mut buffer_owned = buffer.lock().await;
        if buffer_owned.is_empty() {
            drop(buffer_owned);
            continue;
        }

        for chunk in buffer_owned.chunks(MAX_BATCH_SIZE) {
            for event in chunk {
                match event {
                    MetricEvent::IngestedEvent {
                        timestamp,
                        count,
                        bytes,
                        latency,
                        route,
                        method,
                        topic,
                    } => {
                        let _ = client
                            .post("http://localhost:4000/ingest/IngestEvent/0.0")
                            .json(&json!({
                                "timestamp": timestamp,
                                "count": count,
                                "bytes": bytes,
                                "latency": latency.as_secs_f64(),
                                "route": route.as_os_str().to_str().unwrap(),
                                "method": method,
                                "topic": topic,
                            }))
                            .send()
                            .await;
                    }
                    MetricEvent::ConsumedEvent {
                        timestamp,
                        count,
                        latency,
                        bytes,
                        route,
                        method,
                    } => {
                        let _ = client
                            .post("http://localhost:4000/ingest/ConsumptionEvent/0.0")
                            .json(&json!({
                                "timestamp": timestamp,
                                "count": count,
                                "latency": latency.as_secs_f64(),
                                "bytes": bytes,
                                "route": route.as_os_str().to_str().unwrap(),
                                "method": method,
                            }))
                            .send()
                            .await;
                    }
                    MetricEvent::StreamingFunctionEvent {
                        timestamp,
                        count_in,
                        count_out,
                        bytes,
                        function_name,
                    } => {
                        let _ = client
                            .post("http://localhost:4000/ingest/StreamingFunctionEvent/0.0")
                            .json(&json!({
                                "timestamp": timestamp,
                                "count_in": count_in,
                                "count_out": count_out,
                                "bytes": bytes,
                                "function_name": function_name,
                            }))
                            .send()
                            .await;
                    }
                    MetricEvent::TopicToOLAPEvent {
                        timestamp,
                        count,
                        bytes,
                        consumer_group,
                        topic_name,
                    } => {
                        let _ = client
                            .post("http://localhost:4000/ingest/TopicToOLAPEvent/0.0")
                            .json(&json!({
                                "timestamp": timestamp,
                                "count": count,
                                "bytes": bytes,
                                "consumer_group": consumer_group,
                                "topic_name": topic_name,
                            }))
                            .send()
                            .await;
                    }
                }
            }
        }

        buffer_owned.clear();
        drop(buffer_owned);
    }
}
