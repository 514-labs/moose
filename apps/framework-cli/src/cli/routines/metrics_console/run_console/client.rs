use std::collections::HashMap;

use prometheus_parse::{self, HistogramCount, Sample};
use reqwest;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub struct PathMetricsData {
    pub latency_sum: f64,
    pub request_count: f64,
    pub path: String,
}

pub struct ParsedMetricsData {
    pub average_latency: f64,
    pub total_requests: f64,
    pub paths_data_vec: Vec<PathMetricsData>,
    pub paths_bytes_hashmap: HashMap<String, u64>,
    pub total_bytes_in: u64,
    pub total_bytes_out: u64,
    pub histogram_vec: Vec<Sample>,
    pub kafka_messages_in_total: HashMap<String, (String, f64)>,
    pub kafka_messages_out_total: Vec<(String, String, f64)>,
    pub kafka_bytes_out_total: HashMap<String, (String, u64)>,
    pub streaming_functions_in: HashMap<String, f64>,
    pub streaming_functions_out: HashMap<String, f64>,
    pub streaming_functions_bytes: HashMap<String, u64>,
}

pub async fn getting_metrics_data() -> Result<ParsedMetricsData> {
    let body = reqwest::get("http://localhost:4000/metrics")
        .await
        .unwrap()
        .text();
    let lines: Vec<_> = body
        .await
        .unwrap()
        .lines()
        .map(|s| Ok(s.to_owned()))
        .collect();

    let metrics = prometheus_parse::Scrape::parse(lines.into_iter())?;

    let metrics_vec = metrics.samples;

    let mut average_latency: f64 = 0.0;
    let mut total_requests: f64 = 0.0;
    let mut paths_data_vec: Vec<PathMetricsData> = vec![];
    let mut total_bytes_in: u64 = 0;
    let mut total_bytes_out: u64 = 0;
    let mut paths_bytes_hashmap: HashMap<String, u64> = HashMap::new();
    let mut kafka_messages_in_total: HashMap<String, (String, f64)> = HashMap::new();
    let mut kafka_messages_out_total: Vec<(String, String, f64)> = vec![];
    let mut kafka_bytes_out_total: HashMap<String, (String, u64)> = HashMap::new();
    let mut streaming_functions_in: HashMap<String, f64> = HashMap::new();
    let mut streaming_functions_out: HashMap<String, f64> = HashMap::new();
    let mut streaming_functions_bytes: HashMap<String, u64> = HashMap::new();

    let mut i = 0;
    while i < metrics_vec.len() {
        if &metrics_vec[i].metric == "total_latency_sum" {
            let value = match &metrics_vec[i].value {
                prometheus_parse::Value::Histogram(v) => v[0].count,
                prometheus_parse::Value::Untyped(v) => *v,
                _ => 0.0,
            };
            average_latency = value;
        }
        if &metrics_vec[i].metric == "total_latency_count" {
            let value = match &metrics_vec[i].value {
                prometheus_parse::Value::Histogram(v) => v[0].count,
                prometheus_parse::Value::Untyped(v) => *v,
                _ => 0.0,
            };
            total_requests = value;
            average_latency = (average_latency / value) * 1000.0;
        }

        i += 1;
    }

    let mut j = 0;

    while j < metrics_vec.len() {
        if metrics_vec[j].metric == "latency_sum" {
            let sum_value = match &metrics_vec[j].value {
                prometheus_parse::Value::Histogram(v) => v[0].count,
                prometheus_parse::Value::Untyped(v) => *v,
                _ => 0.0,
            };
            let count_value = match &metrics_vec[j + 1].value {
                prometheus_parse::Value::Histogram(v) => v[0].count,
                prometheus_parse::Value::Untyped(v) => *v,
                _ => 0.0,
            };
            paths_data_vec.push(PathMetricsData {
                latency_sum: sum_value,
                request_count: count_value,
                path: (metrics_vec[j].labels["path"]).to_string(),
            });
        } else if metrics_vec[j].metric.starts_with("bytes") {
            let value: f64 = match &metrics_vec[j].value {
                prometheus_parse::Value::Counter(v) => *v,
                prometheus_parse::Value::Untyped(v) => *v,
                _ => 0.0,
            };

            paths_bytes_hashmap.insert((metrics_vec[j].labels["path"]).to_string(), value as u64);

            if metrics_vec[j].labels["path"].starts_with("ingest/") {
                total_bytes_in += value as u64;
            } else {
                total_bytes_out += value as u64;
            }
        } else if metrics_vec[j].metric == "messages_in_total" {
            let value: f64 = match &metrics_vec[j].value {
                prometheus_parse::Value::Counter(v) => *v,
                prometheus_parse::Value::Untyped(v) => *v,
                _ => 0.0,
            };

            let topic = metrics_vec[j].labels["topic"].to_string();
            let path = metrics_vec[j].labels["path"].to_string();

            kafka_messages_in_total.insert(path, (topic, value));
        } else if metrics_vec[j].metric == "messages_out_total" {
            let value: f64 = match &metrics_vec[j].value {
                prometheus_parse::Value::Counter(v) => *v,
                prometheus_parse::Value::Untyped(v) => *v,
                _ => 0.0,
            };

            let consumer_group = metrics_vec[j].labels["consumer_group"].to_string();
            let topic = metrics_vec[j].labels["topic"].to_string();

            kafka_messages_out_total.push((topic, consumer_group, value));
        } else if metrics_vec[j].metric == "kafka_clickhouse_sync_bytes_out_total" {
            let value: f64 = match &metrics_vec[j].value {
                prometheus_parse::Value::Counter(v) => *v,
                prometheus_parse::Value::Untyped(v) => *v,
                _ => 0.0,
            };

            let consumer_group = metrics_vec[j].labels["consumer_group"].to_string();
            let topic = metrics_vec[j].labels["topic"].to_string();

            kafka_bytes_out_total.insert(topic, (consumer_group, value as u64));
        } else if &metrics_vec[j].metric == "streaming_functions_in" {
            let value = match &metrics_vec[j].value {
                prometheus_parse::Value::Gauge(v) => v,
                prometheus_parse::Value::Untyped(v) => v,
                _ => &0.0,
            };
            streaming_functions_in.insert(metrics_vec[j].labels["path"].to_string(), *value);
        } else if &metrics_vec[j].metric == "streaming_functions_out" {
            let value = match &metrics_vec[j].value {
                prometheus_parse::Value::Gauge(v) => v,
                prometheus_parse::Value::Untyped(v) => v,
                _ => &0.0,
            };
            streaming_functions_out.insert(metrics_vec[j].labels["path"].to_string(), *value);
        } else if &metrics_vec[j].metric == "streaming_functions_bytes" {
            let value = match &metrics_vec[j].value {
                prometheus_parse::Value::Gauge(v) => v,
                prometheus_parse::Value::Untyped(v) => v,
                _ => &0.0,
            };
            streaming_functions_bytes
                .insert(metrics_vec[j].labels["path"].to_string(), *value as u64);
        }

        j += 1;
    }

    let parsed_data = ParsedMetricsData {
        average_latency,
        total_requests,
        paths_data_vec,
        paths_bytes_hashmap,
        total_bytes_in,
        total_bytes_out,
        histogram_vec: metrics_vec,
        kafka_messages_in_total,
        kafka_messages_out_total,
        kafka_bytes_out_total,
        streaming_functions_in,
        streaming_functions_out,
        streaming_functions_bytes,
    };

    Ok(parsed_data)
}

pub fn parsing_histogram_data(
    path: String,
    metrics_vec: Vec<prometheus_parse::Sample>,
) -> Option<Vec<HistogramCount>> {
    let mut i = 0;

    while i < metrics_vec.len() {
        let labels = &metrics_vec[i].labels.clone();
        let paths = labels.get("path");

        let unwrapped_path = paths.unwrap_or_default();
        if *unwrapped_path == path && !unwrapped_path.is_empty() {
            if let prometheus_parse::Value::Histogram(v) = &metrics_vec[i].value {
                return Some(v.to_owned());
            }
        }

        i += 1;
    }
    None
}
