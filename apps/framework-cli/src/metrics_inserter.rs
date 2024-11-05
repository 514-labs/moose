use crate::cli::settings::user_directory;
use crate::metrics::MetricEvent;
use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::time;

const METRICS_DIR: &str = "metrics";
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
        write_metrics_to_file: bool,
    ) -> Self {
        let buffer = Arc::new(Mutex::new(Vec::new()));

        tokio::spawn(flush(
            buffer.clone(),
            metric_labels.clone(),
            metric_endpoints.clone(),
            write_metrics_to_file,
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
    write_metrics_to_file: bool,
) {
    let mut interval = time::interval(Duration::from_secs(MAX_FLUSH_INTERVAL_SECONDS));
    let client = Client::new();

    loop {
        interval.tick().await;
        let mut buffer_owned = buffer.lock().await;
        if buffer_owned.is_empty() {
            continue;
        }

        if write_metrics_to_file {
            clean_old_metric_files();
        }

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

            if write_metrics_to_file {
                if let Err(e) = write_events_to_file(event_type, &events).await {
                    eprintln!("Failed to write metrics to file: {}", e);
                }
            } else {
                let _ = client.post(route).json(&events).send().await;
            }
        }

        buffer_owned.clear();
    }
}

async fn write_events_to_file(
    event_type: &str,
    events: &[serde_json::Value],
) -> anyhow::Result<()> {
    let dir_path = user_directory().join(METRICS_DIR).join(event_type);
    fs::create_dir_all(&dir_path).await?;

    let date_str = Utc::now().format("%Y-%m-%d").to_string();
    let file_path = dir_path.join(format!("{}.txt", date_str));

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file_path)
        .await?;

    let mut buffer = String::new();
    for event in events.iter() {
        let event_str = serde_json::to_string(event)?;
        buffer.push_str(&event_str);
        buffer.push('\n');
    }

    file.write_all(buffer.as_bytes()).await?;

    Ok(())
}

fn clean_old_metric_files() {
    let cut_off = SystemTime::now() - Duration::from_secs(7 * 24 * 60 * 60);
    let metrics_dir = user_directory().join(METRICS_DIR);

    if let Ok(dir) = metrics_dir.read_dir() {
        for entry in dir.flatten() {
            let sub_dir = match entry.path().read_dir() {
                Ok(sub_dir) => sub_dir,
                Err(_) => {
                    eprintln!("Failed to read subdirectory: {:?}", entry.path());
                    continue;
                }
            };

            for file_entry in sub_dir.flatten() {
                if file_entry
                    .path()
                    .extension()
                    .map_or(false, |ext| ext == "txt")
                {
                    match file_entry.metadata().and_then(|md| md.modified()) {
                        Ok(t) if t < cut_off => {
                            if let Err(e) = std::fs::remove_file(file_entry.path()) {
                                eprintln!("Failed to remove file {:?}: {}", file_entry.path(), e);
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!(
                                "Failed to read modification time for {:?}. {}",
                                file_entry.path(),
                                e
                            );
                        }
                    }
                }
            }
        }
    } else {
        eprintln!("Failed to read metrics directory");
    }
}
