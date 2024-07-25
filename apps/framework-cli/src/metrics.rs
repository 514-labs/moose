use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::{
    encoding::{text::encode, EncodeLabelSet},
    metrics::histogram::Histogram,
    registry::Registry,
};
use std::{path::PathBuf, sync::Arc, time::Duration};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MetricsErrors {
    #[error("Failed to get metrics data")]
    OneShotError(#[from] tokio::sync::oneshot::error::RecvError),
}

pub enum MetricsMessage {
    GetMetricsRegistryAsString(tokio::sync::oneshot::Sender<String>),
    HTTPLatency((PathBuf, Duration, String)),
    PutNumberOfBytesIn(PathBuf, u64),
    PutNumberOfBytesOut(PathBuf, u64),
    PutNumberOfMessagesIn(String, String),
    PutNumberOfMessagesOut(String, String),
    PutKafkaClickhouseSyncBytesOut(String, String, u64),
    PutStreamingFunctionMessagesIn(String, u64),
    PutStreamingFunctionMessagesOut(String, u64),
    PutStreamingFunctionBytes(String, u64),
}

#[derive(Clone)]
pub struct Metrics {
    pub tx: tokio::sync::mpsc::Sender<MetricsMessage>,
}

pub struct Statistics {
    pub total_latency_histogram: Histogram,
    pub histogram_family: Family<HistogramLabels, Histogram>,
    pub bytes_in_family: Family<BytesCounterLabels, Counter>,
    pub bytes_out_family: Family<BytesCounterLabels, Counter>,
    pub messages_in_family: Family<MessagesInCounterLabels, Counter>,
    pub messages_out_family: Family<MessagesOutCounterLabels, Counter>,
    pub kafka_clickhouse_sync_bytes_out_family: Family<MessagesOutCounterLabels, Counter>,
    pub streaming_functions_in_family: Family<StreamingFunctionMessagesCounterLabels, Gauge>,
    pub streaming_functions_out_family: Family<StreamingFunctionMessagesCounterLabels, Gauge>,
    pub streaming_functions_bytes_family: Family<StreamingFunctionMessagesCounterLabels, Counter>,
    pub registry: Option<Registry>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HistogramLabels {
    method: String,
    path: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct BytesCounterLabels {
    path: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct StreamingFunctionMessagesCounterLabels {
    path: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct MessagesInCounterLabels {
    path: String,
    topic: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct MessagesOutCounterLabels {
    consumer_group: String,
    topic: String,
}

impl Metrics {
    pub fn new() -> (Metrics, tokio::sync::mpsc::Receiver<MetricsMessage>) {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let metrics = Metrics { tx };
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
        self: &Arc<Metrics>,
        mut rx: tokio::sync::mpsc::Receiver<MetricsMessage>,
    ) {
        let mut data = Statistics {
            total_latency_histogram: Histogram::new(
                [
                    0.001, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0,
                    240.0,
                ]
                .into_iter(),
            ),
            histogram_family: Family::<HistogramLabels, Histogram>::new_with_constructor(|| {
                Histogram::new(
                    [
                        0.001, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0,
                        240.0,
                    ]
                    .into_iter(),
                )
            }),
            bytes_in_family: Family::<BytesCounterLabels, Counter>::new_with_constructor(|| {
                Counter::default()
            }),
            bytes_out_family: Family::<BytesCounterLabels, Counter>::new_with_constructor(|| {
                Counter::default()
            }),
            messages_in_family: Family::<MessagesInCounterLabels, Counter>::new_with_constructor(
                Counter::default,
            ),
            messages_out_family: Family::<MessagesOutCounterLabels, Counter>::new_with_constructor(
                Counter::default,
            ),
            kafka_clickhouse_sync_bytes_out_family:
                Family::<MessagesOutCounterLabels, Counter>::new_with_constructor(Counter::default),
            streaming_functions_in_family:
                Family::<StreamingFunctionMessagesCounterLabels, Gauge>::new_with_constructor(
                    Gauge::default,
                ),
            streaming_functions_out_family:
                Family::<StreamingFunctionMessagesCounterLabels, Gauge>::new_with_constructor(
                    Gauge::default,
                ),
            streaming_functions_bytes_family: Family::<
                StreamingFunctionMessagesCounterLabels,
                Counter,
            >::new_with_constructor(Counter::default),
            registry: Some(Registry::default()),
        };
        let mut new_registry = data.registry.unwrap();
        new_registry.register(
            "total_latency",
            "Total latency of HTTP requests",
            data.total_latency_histogram.clone(),
        );
        new_registry.register(
            "latency",
            "Latency of HTTP requests",
            data.histogram_family.clone(),
        );
        new_registry.register(
            "bytes_in",
            "Bytes received through ingest endpoints",
            data.bytes_in_family.clone(),
        );
        new_registry.register(
            "bytes_out",
            "Bytes sent out through consumption endpoints",
            data.bytes_out_family.clone(),
        );
        new_registry.register(
            "messages_in",
            "Messages sent to kafka stream",
            data.messages_in_family.clone(),
        );
        new_registry.register(
            "messages_out",
            "Messages received from kafka stream",
            data.messages_out_family.clone(),
        );

        new_registry.register(
            "streaming_functions_in",
            "Messages sent from one data model to another using kafka stream",
            data.streaming_functions_in_family.clone(),
        );
        new_registry.register(
            "streaming_functions_out",
            "Messages received from one data model to another using kafka stream",
            data.streaming_functions_out_family.clone(),
        );

        new_registry.register(
<<<<<<< HEAD
            "kafka_clickhouse_sync_bytes_out",
            "Bytes sent to clickhouse",
            data.kafka_clickhouse_sync_bytes_out_family.clone(),
=======
            "streaming_functions_bytes",
            "Bytes sent from one data model to another using kafka stream",
            data.streaming_functions_bytes_family.clone(),
>>>>>>> 0131638 (streaming functions bytes per second added)
        );

        data.registry = Some(new_registry);

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    MetricsMessage::GetMetricsRegistryAsString(v) => {
                        let _ = v.send(formatted_registry(&data.registry.as_ref()).await);
                    }
                    MetricsMessage::HTTPLatency((path, duration, method)) => {
                        data.histogram_family
                            .get_or_create(&HistogramLabels {
                                method,
                                path: path.into_os_string().to_str().unwrap().to_string(),
                            })
                            .observe(duration.as_secs_f64());
                        data.total_latency_histogram.observe(duration.as_secs_f64())
                    }
                    MetricsMessage::PutNumberOfBytesIn(path, number_of_bytes) => {
                        data.bytes_in_family
                            .get_or_create(&BytesCounterLabels {
                                path: path.clone().into_os_string().to_str().unwrap().to_string(),
                            })
                            .inc_by(number_of_bytes);
                    }
                    MetricsMessage::PutNumberOfBytesOut(path, number_of_bytes) => {
                        data.bytes_out_family
                            .get_or_create(&BytesCounterLabels {
                                path: path.clone().into_os_string().to_str().unwrap().to_string(),
                            })
                            .inc_by(number_of_bytes);
                    }
                    MetricsMessage::PutNumberOfMessagesIn(path, topic) => {
                        data.messages_in_family
                            .get_or_create(&MessagesInCounterLabels { path, topic })
                            .inc();
                    }
                    MetricsMessage::PutNumberOfMessagesOut(consumer_group, topic) => {
                        data.messages_out_family
                            .get_or_create(&MessagesOutCounterLabels {
                                consumer_group,
                                topic,
                            })
                            .inc();
                    }
                    MetricsMessage::PutStreamingFunctionMessagesIn(path, count) => {
                        data.streaming_functions_in_family
                            .get_or_create(&StreamingFunctionMessagesCounterLabels { path })
                            .set(count as i64);
                    }
                    MetricsMessage::PutStreamingFunctionMessagesOut(path, count) => {
                        data.streaming_functions_out_family
                            .get_or_create(&StreamingFunctionMessagesCounterLabels { path })
                            .set(count as i64);
                    }
                    MetricsMessage::PutKafkaClickhouseSyncBytesOut(
                        consumer_group,
                        topic,
                        number_of_bytes,
                    ) => {
                        data.kafka_clickhouse_sync_bytes_out_family
                            .get_or_create(&MessagesOutCounterLabels {
                                consumer_group,
                                topic,
                            })
                            .inc_by(number_of_bytes);
                    MetricsMessage::PutStreamingFunctionBytes(path, count) => {
                        data.streaming_functions_bytes_family
                            .get_or_create(&StreamingFunctionMessagesCounterLabels { path })
                            .inc_by(count);
                    }
                };
            }
        });
    }
}

pub async fn formatted_registry(data: &Option<&Registry>) -> String {
    let mut buffer = String::new();
    let _ = encode(&mut buffer, data.as_ref().unwrap());
    buffer
}
