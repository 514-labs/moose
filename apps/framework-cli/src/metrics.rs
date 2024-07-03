use prometheus_client::metrics::family::Family;
use prometheus_client::{
    encoding::{text::encode, EncodeLabelSet, EncodeLabelValue},
    metrics::histogram::Histogram,
    registry::Registry,
};
use std::{path::PathBuf, sync::Arc, time::Duration};

pub enum MetricsMessage {
    GetMetricsRegistryAsString(tokio::sync::oneshot::Sender<String>),
    HTTPLatency((PathBuf, Duration, Method)),
}

#[derive(Clone)]
pub struct Metrics {
    pub tx: tokio::sync::mpsc::Sender<MetricsMessage>,
}

pub struct Statistics {
    pub histogram_family: Family<Labels, Histogram>,
    pub registry: Option<Registry>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct Labels {
    method: Method,
    path: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelValue)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    CONNECT,
    PATCH,
    TRACE,
    OTHER,
}

impl Metrics {
    pub async fn send_data(&self, data: MetricsMessage) {
        let _ = self.tx.send(data).await;
    }

    pub async fn receive_data(&self) -> String {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel::<String>();
        self.tx
            .send(MetricsMessage::GetMetricsRegistryAsString(resp_tx))
            .await
            .unwrap();

        let res = resp_rx.await;

        res.unwrap()
    }

    pub async fn start_listening_to_metrics(
        self: Arc<Metrics>,
        mut rx: tokio::sync::mpsc::Receiver<MetricsMessage>,
    ) {
        let mut data = Statistics {
            histogram_family: Family::<Labels, Histogram>::new_with_constructor(|| {
                Histogram::new(
                    [
                        0.001, 0.01, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0,
                        240.0,
                    ]
                    .into_iter(),
                )
            }),
            registry: Some(Registry::default()),
        };
        let mut new_registry = data.registry.unwrap();
        new_registry.register(
            "latency",
            "Latency of HTTP requests",
            data.histogram_family.clone(),
        );

        data.registry = Some(new_registry);

        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    MetricsMessage::GetMetricsRegistryAsString(v) => {
                        let _ = v.send(formatted_registry(&data.registry.as_ref()).await);
                    }
                    MetricsMessage::HTTPLatency((path, duration, method)) => data
                        .histogram_family
                        .get_or_create(&Labels {
                            method,
                            path: path.into_os_string().to_str().unwrap().to_string(),
                        })
                        .observe(duration.as_secs_f64()),
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
