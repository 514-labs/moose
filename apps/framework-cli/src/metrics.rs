use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
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
    GetNumberOfBytesIn(PathBuf, u64),
    GetNumberOfBytesOut(PathBuf, u64),
}

#[derive(Clone)]
pub struct Metrics {
    pub tx: tokio::sync::mpsc::Sender<MetricsMessage>,
}

pub struct Statistics {
    pub total_latency_histogram: Histogram,
    pub histogram_family: Family<HistogramLabels, Histogram>,
    pub bytes_in_family: Family<CounterLabels, Counter>,
    pub bytes_out_family: Family<CounterLabels, Counter>,
    pub registry: Option<Registry>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HistogramLabels {
    method: String,
    path: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct CounterLabels {
    path: String,
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
            bytes_in_family: Family::<CounterLabels, Counter>::new_with_constructor(|| {
                Counter::default()
            }),
            bytes_out_family: Family::<CounterLabels, Counter>::new_with_constructor(|| {
                Counter::default()
            }),
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
                    MetricsMessage::GetNumberOfBytesIn(path, number_of_bytes) => {
                        data.bytes_in_family
                            .get_or_create(&CounterLabels {
                                path: path.clone().into_os_string().to_str().unwrap().to_string(),
                            })
                            .inc_by(number_of_bytes);
                    }
                    MetricsMessage::GetNumberOfBytesOut(path, number_of_bytes) => {
                        data.bytes_out_family
                            .get_or_create(&CounterLabels {
                                path: path.clone().into_os_string().to_str().unwrap().to_string(),
                            })
                            .inc_by(number_of_bytes);
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
