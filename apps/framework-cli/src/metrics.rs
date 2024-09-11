use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::{
    encoding::{text::encode, EncodeLabelSet},
    metrics::histogram::Histogram,
    registry::Registry,
};
use serde_json::json;
use serde_json::Value;
use std::env;
use std::sync::Arc;
use std::{path::PathBuf, time::Duration};
use tokio::time;

use chrono::Utc;

use crate::utilities::constants::{self, CONTEXT, CTX_SESSION_ID};
use crate::utilities::decode_object;
const DEFAULT_ANONYMOUS_METRICS_URL: &str =
    "https://moosefood.514.dev/ingest/MooseSessionTelemetry/0.6";
lazy_static::lazy_static! {
    static ref ANONYMOUS_METRICS_URL: String = env::var("MOOSE_METRICS_DEST")
        .unwrap_or_else(|_| DEFAULT_ANONYMOUS_METRICS_URL.to_string());
}
const ANONYMOUS_METRICS_REPORTING_INTERVAL: Duration = Duration::from_secs(10);
pub const TOTAL_LATENCY: &str = "moose_total_latency";
pub const LATENCY: &str = "moose_latency";
pub const INGESTED_BYTES: &str = "moose_ingested_bytes";
pub const CONSUMED_BYTES: &str = "moose_consumed_bytes";
pub const HTTP_TO_TOPIC_EVENT_COUNT: &str = "moose_http_to_topic_event_count";
pub const TOPIC_TO_OLAP_EVENT_COUNT: &str = "moose_topic_to_olap_event_count";
pub const TOPIC_TO_OLAP_BYTE_COUNT: &str = "moose_topic_to_olap_bytes_count";
pub const STREAMING_FUNCTION_EVENT_INPUT_COUNT: &str =
    "moose_streaming_functions_events_input_count";
pub const STREAMING_FUNCTION_EVENT_OUPUT_COUNT: &str =
    "moose_streaming_functions_events_output_count";
pub const STREAMING_FUNCTION_PROCESSED_BYTE_COUNT: &str =
    "moose_streaming_functions_processed_byte_count";

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MetricsErrors {
    #[error("Failed to get metrics data")]
    OneShotError(#[from] tokio::sync::oneshot::error::RecvError),
}

pub enum MetricsMessage {
    GetMetricsRegistryAsString(tokio::sync::oneshot::Sender<String>),
    HTTPLatency {
        path: PathBuf,
        method: String,
        duration: Duration,
    },
    PutIngestedBytesCount {
        route: PathBuf,
        method: String,
        bytes_count: u64,
    },
    PutConsumedBytesCount {
        route: PathBuf,
        method: String,
        bytes_count: u64,
    },
    PutHTTPToTopicEventCount {
        topic_name: String,
        route: PathBuf,
        method: String,
        count: u64,
    },
    PutTopicToOLAPEventCount {
        consumer_group: String,
        topic_name: String,
        count: u64,
    },
    PutTopicToOLAPBytesCount {
        consumer_group: String,
        topic_name: String,
        bytes_count: u64,
    },
    PutStreamingFunctionMessagesIn {
        function_name: String,
        count: u64,
    },
    PutStreamingFunctionMessagesOut {
        function_name: String,
        count: u64,
    },
    PutStreamingFunctionBytes {
        function_name: String,
        bytes_count: u64,
    },
    PutBlockCount {
        count: i64,
    },
}

#[derive(Clone)]
pub struct TelemetryMetadata {
    pub anonymous_telemetry_enabled: bool,
    pub machine_id: String,
    pub is_moose_developer: bool,
    pub metric_labels: Option<String>,
    pub is_production: bool,
    pub project_name: String,
}

#[derive(Clone)]
pub struct Metrics {
    pub tx: tokio::sync::mpsc::Sender<MetricsMessage>,
    telemetry_metadata: TelemetryMetadata,
}

pub struct Statistics {
    pub http_latency_histogram_aggregate: Histogram,
    pub http_latency_histogram: Family<HTTPLabel, Histogram>,
    pub http_ingested_latency_sum_ms: Counter,
    pub http_ingested_request_count: Counter,
    pub http_ingested_total_bytes: Counter,
    pub http_ingested_bytes: Family<HTTPLabel, Counter>,
    pub http_consumed_request_count: Counter,
    pub http_consumed_latency_sum_ms: Counter,
    pub http_consumed_bytes: Family<HTTPLabel, Counter>,
    pub http_to_topic_event_count: Family<MessagesInCounterLabels, Counter>,
    pub blocks_count: Gauge,
    pub topic_to_olap_event_count: Family<MessagesOutCounterLabels, Counter>,
    pub topic_to_olap_event_total_count: Counter,
    pub topic_to_olap_bytes_count: Family<MessagesOutCounterLabels, Counter>,
    pub topic_to_olap_bytes_total_count: Counter,
    pub streaming_functions_in_event_count: Family<StreamingFunctionMessagesCounterLabels, Counter>,
    pub streaming_functions_out_event_count:
        Family<StreamingFunctionMessagesCounterLabels, Counter>,
    pub streaming_functions_processed_bytes_count:
        Family<StreamingFunctionMessagesCounterLabels, Counter>,
    pub streaming_functions_in_event_total_count: Counter,
    pub streaming_functions_out_event_total_count: Counter,
    pub streaming_functions_processed_bytes_total_count: Counter,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HTTPLabel {
    method: String,
    path: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct StreamingFunctionMessagesCounterLabels {
    function_name: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct MessagesInCounterLabels {
    path: String,
    method: String,
    topic_name: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct MessagesOutCounterLabels {
    consumer_group: String,
    topic_name: String,
}

impl Metrics {
    pub fn new(
        telemetry_metadata: TelemetryMetadata,
    ) -> (Metrics, tokio::sync::mpsc::Receiver<MetricsMessage>) {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let metrics = Metrics {
            tx,
            telemetry_metadata,
        };
        (metrics, rx)
    }

    pub async fn send_metric(&self, data: MetricsMessage) {
        let _ = self.tx.send(data).await;
    }

    pub async fn get_prometheus_metrics_string(
        &self,
    ) -> Result<std::string::String, MetricsErrors> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel::<String>();
        let _ = self
            .tx
            .send(MetricsMessage::GetMetricsRegistryAsString(resp_tx))
            .await;

        Ok(resp_rx.await?)
    }

    pub async fn start_listening_to_metrics(
        &self,
        mut rx: tokio::sync::mpsc::Receiver<MetricsMessage>,
    ) {
        let data = Arc::new(Statistics {
            http_ingested_request_count: Counter::default(),
            http_ingested_total_bytes: Counter::default(),
            http_ingested_latency_sum_ms: Counter::default(),
            http_consumed_latency_sum_ms: Counter::default(),
            http_consumed_request_count: Counter::default(),
            streaming_functions_in_event_total_count: Counter::default(),
            streaming_functions_out_event_total_count: Counter::default(),
            streaming_functions_processed_bytes_total_count: Counter::default(),
            topic_to_olap_event_total_count: Counter::default(),
            blocks_count: Gauge::default(),
            topic_to_olap_bytes_total_count: Counter::default(),
            http_latency_histogram_aggregate: Histogram::new(
                [
                    0.001, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0,
                    240.0,
                ]
                .into_iter(),
            ),
            http_latency_histogram: Family::<HTTPLabel, Histogram>::new_with_constructor(|| {
                Histogram::new(
                    [
                        0.001, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0,
                        240.0,
                    ]
                    .into_iter(),
                )
            }),
            http_ingested_bytes: Family::<HTTPLabel, Counter>::new_with_constructor(|| {
                Counter::default()
            }),
            http_consumed_bytes: Family::<HTTPLabel, Counter>::new_with_constructor(|| {
                Counter::default()
            }),
            http_to_topic_event_count:
                Family::<MessagesInCounterLabels, Counter>::new_with_constructor(Counter::default),
            topic_to_olap_event_count:
                Family::<MessagesOutCounterLabels, Counter>::new_with_constructor(Counter::default),
            topic_to_olap_bytes_count:
                Family::<MessagesOutCounterLabels, Counter>::new_with_constructor(Counter::default),
            streaming_functions_in_event_count: Family::<
                StreamingFunctionMessagesCounterLabels,
                Counter,
            >::new_with_constructor(
                Counter::default
            ),
            streaming_functions_out_event_count: Family::<
                StreamingFunctionMessagesCounterLabels,
                Counter,
            >::new_with_constructor(
                Counter::default
            ),
            streaming_functions_processed_bytes_count: Family::<
                StreamingFunctionMessagesCounterLabels,
                Counter,
            >::new_with_constructor(
                Counter::default
            ),
        });

        let mut registry = Registry::default();

        registry.register(
            TOTAL_LATENCY,
            "Total latency of HTTP requests",
            data.http_latency_histogram_aggregate.clone(),
        );
        registry.register(
            LATENCY,
            "Latency of HTTP requests",
            data.http_latency_histogram.clone(),
        );
        registry.register(
            INGESTED_BYTES,
            "Bytes received through ingest endpoints",
            data.http_ingested_bytes.clone(),
        );
        registry.register(
            CONSUMED_BYTES,
            "Bytes sent out through consumption endpoints",
            data.http_consumed_bytes.clone(),
        );
        registry.register(
            HTTP_TO_TOPIC_EVENT_COUNT,
            "Messages sent to kafka stream",
            data.http_to_topic_event_count.clone(),
        );
        registry.register(
            TOPIC_TO_OLAP_EVENT_COUNT,
            "Messages received from kafka stream",
            data.topic_to_olap_event_count.clone(),
        );

        registry.register(
            STREAMING_FUNCTION_EVENT_INPUT_COUNT,
            "Messages sent from one data model to another using kafka stream",
            data.streaming_functions_in_event_count.clone(),
        );
        registry.register(
            STREAMING_FUNCTION_EVENT_OUPUT_COUNT,
            "Messages received from one data model to another using kafka stream",
            data.streaming_functions_out_event_count.clone(),
        );

        registry.register(
            TOPIC_TO_OLAP_BYTE_COUNT,
            "Bytes sent to clickhouse",
            data.topic_to_olap_bytes_count.clone(),
        );
        registry.register(
            STREAMING_FUNCTION_PROCESSED_BYTE_COUNT,
            "Bytes sent from one data model to another using kafka stream",
            data.streaming_functions_processed_bytes_count.clone(),
        );

        let cloned_data_ref = data.clone();

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    MetricsMessage::GetMetricsRegistryAsString(v) => {
                        let _ = v.send(formatted_registry(&registry).await);
                    }
                    MetricsMessage::HTTPLatency {
                        path,
                        duration,
                        method,
                    } => {
                        data.http_latency_histogram
                            .get_or_create(&HTTPLabel {
                                method,
                                path: path.clone().into_os_string().to_str().unwrap().to_string(),
                            })
                            .observe(duration.as_secs_f64());
                        data.http_latency_histogram_aggregate
                            .observe(duration.as_secs_f64());
                        if path.starts_with("ingest") {
                            data.http_ingested_latency_sum_ms
                                .inc_by(duration.as_millis() as u64);
                        } else {
                            data.http_consumed_latency_sum_ms
                                .inc_by(duration.as_millis() as u64);
                        }
                    }
                    MetricsMessage::PutIngestedBytesCount {
                        route: path,
                        bytes_count,
                        method,
                    } => {
                        data.http_ingested_bytes
                            .get_or_create(&HTTPLabel {
                                method,
                                path: path.clone().into_os_string().to_str().unwrap().to_string(),
                            })
                            .inc_by(bytes_count);
                        data.http_ingested_request_count.inc();
                        data.http_ingested_total_bytes.inc_by(bytes_count);
                    }
                    MetricsMessage::PutConsumedBytesCount {
                        route: path,
                        bytes_count,
                        method,
                    } => {
                        data.http_consumed_bytes
                            .get_or_create(&HTTPLabel {
                                method,
                                path: path.clone().into_os_string().to_str().unwrap().to_string(),
                            })
                            .inc_by(bytes_count);
                    }
                    MetricsMessage::PutHTTPToTopicEventCount {
                        route: path,
                        topic_name,
                        method,
                        count,
                    } => {
                        data.http_to_topic_event_count
                            .get_or_create(&MessagesInCounterLabels {
                                path: path.clone().into_os_string().to_str().unwrap().to_string(),
                                topic_name,
                                method,
                            })
                            .inc_by(count);
                    }
                    MetricsMessage::PutTopicToOLAPEventCount {
                        consumer_group,
                        topic_name,
                        count,
                    } => {
                        data.topic_to_olap_event_count
                            .get_or_create(&MessagesOutCounterLabels {
                                consumer_group,
                                topic_name,
                            })
                            .inc_by(count);
                        data.topic_to_olap_event_total_count.inc_by(count);
                    }
                    MetricsMessage::PutStreamingFunctionMessagesIn {
                        function_name,
                        count,
                    } => {
                        data.streaming_functions_in_event_count
                            .get_or_create(&StreamingFunctionMessagesCounterLabels {
                                function_name,
                            })
                            .inc_by(count);
                        data.streaming_functions_in_event_total_count.inc_by(count);
                    }
                    MetricsMessage::PutStreamingFunctionMessagesOut {
                        function_name,
                        count,
                    } => {
                        data.streaming_functions_out_event_count
                            .get_or_create(&StreamingFunctionMessagesCounterLabels {
                                function_name,
                            })
                            .inc_by(count);
                        data.streaming_functions_out_event_total_count.inc_by(count);
                    }
                    MetricsMessage::PutTopicToOLAPBytesCount {
                        consumer_group,
                        topic_name,
                        bytes_count,
                    } => {
                        data.topic_to_olap_bytes_count
                            .get_or_create(&MessagesOutCounterLabels {
                                consumer_group,
                                topic_name,
                            })
                            .inc_by(bytes_count);
                        data.topic_to_olap_bytes_total_count.inc_by(bytes_count);
                    }
                    MetricsMessage::PutStreamingFunctionBytes {
                        function_name,
                        bytes_count: count,
                    } => {
                        data.streaming_functions_processed_bytes_count
                            .get_or_create(&StreamingFunctionMessagesCounterLabels {
                                function_name,
                            })
                            .inc_by(count);
                        data.streaming_functions_processed_bytes_total_count
                            .inc_by(count);
                    }
                    MetricsMessage::PutBlockCount { count } => {
                        data.blocks_count.set(count);
                    }
                };
            }
        });

        let cloned_metadata = self.telemetry_metadata.clone();

        let metric_labels = match cloned_metadata.metric_labels {
            Some(labels) => match decode_object::decode_base64_to_json(labels.as_str()) {
                Ok(decoded) => decoded,
                Err(e) => {
                    log::warn!("Failed to decode metric labels: {:?}", e);
                    serde_json::Value::Null
                }
            },
            None => serde_json::Value::Null,
        };

        if self.telemetry_metadata.anonymous_telemetry_enabled {
            tokio::spawn(async move {
                let client = reqwest::Client::new();

                let session_start = Utc::now();

                let ip_response = client
                    .get("https://api64.ipify.org?format=text")
                    .timeout(Duration::from_secs(2))
                    .send()
                    .await
                    .ok();

                let ip = if let Some(response) = ip_response {
                    Some(response.text().await.unwrap())
                } else {
                    None
                };

                loop {
                    let _ = time::sleep(ANONYMOUS_METRICS_REPORTING_INTERVAL).await;

                    let session_duration_in_sec = Utc::now()
                        .signed_duration_since(session_start)
                        .num_seconds();

                    let ingested_avg_latency_in_ms =
                        if cloned_data_ref.http_ingested_request_count.get() != 0 {
                            cloned_data_ref.http_ingested_latency_sum_ms.get()
                                / cloned_data_ref.http_ingested_request_count.get()
                        } else {
                            0
                        };

                    let consumed_avg_latency_in_ms =
                        if cloned_data_ref.http_consumed_request_count.get() != 0 {
                            cloned_data_ref.http_consumed_latency_sum_ms.get()
                                / cloned_data_ref.http_consumed_request_count.get()
                        } else {
                            0
                        };

                    let mut telemetry_payload = json!({
                        "timestamp": Utc::now(),
                        "machineId": cloned_metadata.machine_id.clone(),
                        "sequenceId": CONTEXT.get(CTX_SESSION_ID).unwrap(),
                        "project": cloned_metadata.project_name.clone(),
                        "isProd": cloned_metadata.is_production.clone(),
                        "isMooseDeveloper": cloned_metadata.is_moose_developer.clone(),
                        "cliVersion": constants::CLI_VERSION,
                        "sessionDurationInSec": session_duration_in_sec,
                        "ingestedEventsCount": cloned_data_ref.http_ingested_request_count.get(),
                        "ingestedEventsTotalBytes": cloned_data_ref.http_ingested_total_bytes.get(),
                        "ingestAvgLatencyInMs": ingested_avg_latency_in_ms,
                        "consumedRequestCount": cloned_data_ref.http_consumed_request_count.get(),
                        "consumedAvgLatencyInMs": consumed_avg_latency_in_ms,
                        "blocksCount": cloned_data_ref.blocks_count.get(),
                        "streamingToOLAPEventSyncedCount": cloned_data_ref.topic_to_olap_event_total_count.get(),
                        "streamingToOLAPEventSyncedBytesCount": cloned_data_ref.topic_to_olap_bytes_total_count.get(),
                        "streamingFunctionsInputEventsProcessedCount": cloned_data_ref.streaming_functions_in_event_total_count.get(),
                        "streamingFunctionsOutputEventsProcessedCount": cloned_data_ref.streaming_functions_out_event_total_count.get(),
                        "streamingFunctionsEventsProcessedTotalBytes": cloned_data_ref.streaming_functions_processed_bytes_total_count.get(),
                        "ip": ip,
                    });

                    // Merge metric_labels into telemetry_payload
                    if let Some(payload_obj) = telemetry_payload.as_object_mut() {
                        if let Value::Object(labels_obj) = metric_labels.clone() {
                            payload_obj.extend(labels_obj);
                        }
                    }

                    let _ = client
                        .post(ANONYMOUS_METRICS_URL.as_str())
                        .json(&telemetry_payload)
                        .send()
                        .await;
                }
            });
        }
    }
}

pub async fn formatted_registry(data: &Registry) -> String {
    let mut buffer = String::new();
    let _ = encode(&mut buffer, data);
    buffer
}
