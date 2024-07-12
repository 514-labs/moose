use prometheus_parse::{self};
use reqwest;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn getting_metrics_data() -> Result<(f64, f64, Vec<(f64, f64, String)>)> {
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
    let mut paths_latency_average_vec: Vec<(f64, f64, String)> = vec![];

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
            paths_latency_average_vec.push((
                sum_value,
                count_value,
                (metrics_vec[j].labels["path"]).to_string(),
            ));
        }
        j += 1;
    }

    Ok((average_latency, total_requests, paths_latency_average_vec))
}
