use log::{debug, info};
use reqwest::Url;

/// Transforms a ClickHouse connection string to ensure it uses HTTP(S) protocol
/// Handles both clickhouse:// and https:// URLs
pub fn convert_clickhouse_url(conn_str: &str) -> anyhow::Result<Url> {
    let mut url = Url::parse(conn_str)?;

    // Handle clickhouse:// protocol by converting to http(s)
    if url.scheme() == "clickhouse" {
        debug!("Converting clickhouse:// protocol to HTTP(s)");
        let is_secure = match (url.host_str(), url.port()) {
            (_, Some(9000)) => false,
            (_, Some(9440)) => true,
            (Some(host), _) if host == "localhost" || host == "127.0.0.1" => false,
            _ => true,
        };
        let (new_port, new_scheme) = if is_secure {
            (8443, "https")
        } else {
            (8123, "http")
        };

        // Replace clickhouse scheme with http(s)
        url = Url::parse(&conn_str.replacen("clickhouse", new_scheme, 1))?;
        url.set_port(Some(new_port)).unwrap();

        // Move database from path to query parameter if needed
        let path_segments = url.path().split('/').collect::<Vec<&str>>();
        if path_segments.len() == 2 && path_segments[0].is_empty() && !path_segments[1].is_empty() {
            let database = path_segments[1].to_string();
            url.set_path("");
            url.query_pairs_mut().append_pair("database", &database);
        }

        info!(
            "Converted connection string to: {}://{}",
            new_scheme,
            url.host_str().unwrap_or("localhost")
        );
    }

    Ok(url)
}
