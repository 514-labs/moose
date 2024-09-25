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
    pub fn new(
        metric_labels: Option<serde_json::Map<String, serde_json::Value>>,
        metric_endpoints: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Self {
        let buffer = Arc::new(Mutex::new(Vec::new()));

        tokio::spawn(flush(
            buffer.clone(),
            metric_labels.clone(),
            metric_endpoints.clone(),
        ));

        Self { buffer }
    }

    pub async fn insert(&self, event: MetricEvent) -> anyhow::Result<()> {
        let mut buffer = self.buffer.lock().await;
        buffer.push(event);
        Ok(())
    }
}

async fn flush(
    buffer: BatchEvents,
    metric_labels: Option<serde_json::Map<String, serde_json::Value>>,
    metric_endpoints: Option<serde_json::Map<String, serde_json::Value>>,
) {
    let mut interval = time::interval(Duration::from_secs(MAX_FLUSH_INTERVAL_SECONDS));
    let client = Client::new();

    loop {
        interval.tick().await;
        let mut buffer_owned = buffer.lock().await;

        let mut event_groups: std::collections::HashMap<&str, Vec<serde_json::Value>> =
            std::collections::HashMap::new();

        for chunk in buffer_owned.chunks(MAX_BATCH_SIZE) {
            for event in chunk {
                let (event_type, payload) = match event {
                    MetricEvent::IngestedEvent {
                        timestamp,
                        count,
                        bytes,
                        latency,
                        route,
                        method,
                        topic,
                    } => (
                        "IngestEvent",
                        &json!({
                            "timestamp": timestamp,
                            "count": count,
                            "bytes": bytes,
                            "latency": latency.as_secs_f64(),
                            "route": route.clone(),
                            "method": method,
                            "topic": topic,
                        }),
                    ),

                    MetricEvent::ConsumedEvent {
                        timestamp,
                        count,
                        latency,
                        bytes,
                        route,
                        method,
                    } => (
                        "ConsumptionEvent",
                        &json!({
                            "timestamp": timestamp,
                            "count": count,
                            "latency": latency.as_secs_f64(),
                            "bytes": bytes,
                            "route": route.clone(),
                            "method": method,
                        }),
                    ),

                    MetricEvent::StreamingFunctionEvent {
                        timestamp,
                        count_in,
                        count_out,
                        bytes,
                        function_name,
                    } => (
                        "StreamingFunctionEvent",
                        &json!({
                            "timestamp": timestamp,
                            "count_in": count_in,
                            "count_out": count_out,
                            "bytes": bytes,
                            "function_name": function_name,
                        }),
                    ),
                    MetricEvent::TopicToOLAPEvent {
                        timestamp,
                        count,
                        bytes,
                        consumer_group,
                        topic_name,
                    } => (
                        "TopicToOLAPEvent",
                        &json!({
                            "timestamp": timestamp,
                            "count": count,
                            "bytes": bytes,
                            "consumer_group": consumer_group,
                            "topic_name": topic_name,
                        }),
                    ),
                };

                let mut payload = payload.clone();
                let payload_obj = payload.as_object_mut().unwrap();
                if let Some(labels_obj) = &metric_labels {
                    payload_obj.extend(labels_obj.iter().map(|(k, v)| (k.clone(), v.clone())));
                }

                event_groups.entry(event_type).or_default().push(payload);
            }
        }

        for (event_type, events) in event_groups {
            let route = match metric_endpoints
                .as_ref()
                .and_then(|endpoints| endpoints.get(event_type))
                .and_then(|endpoint| endpoint.as_str())
            {
                Some(route) => route,
                None => {
                    eprintln!("Error: No endpoint found for event type: {}", event_type);
                    continue;
                }
            };

            let _ = client.post(route).json(&events).send().await;
        }

        buffer_owned.clear();
    }
}
