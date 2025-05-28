use log::{debug, info};
use reqwest::Url;

/// Transforms a ClickHouse connection string to ensure it uses HTTP(S) protocol
/// Handles both clickhouse:// and https:// URLs
pub fn convert_clickhouse_url_to_http(conn_str: &str) -> anyhow::Result<Url> {
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

/// Alias for backward compatibility
pub fn convert_clickhouse_url(conn_str: &str) -> anyhow::Result<Url> {
    convert_clickhouse_url_to_http(conn_str)
}

/// Transforms HTTP(S) ClickHouse connection string to use native ClickHouse ports
/// This is the inverse of convert_clickhouse_url_to_http
/// Processes URLs that start with http://, https://, or clickhouse://
/// Maps 8443 -> 9440 (https -> secure clickhouse) and 8123 -> 9000 (http -> insecure clickhouse)
pub fn convert_http_to_clickhouse(conn_str: &str) -> anyhow::Result<Url> {
    let mut url = Url::parse(conn_str)?;

    // Handle different URL schemes
    match url.scheme() {
        "http" => {
            // Convert HTTP ports to ClickHouse native ports
            let current_port = url.port();
            let new_port = match current_port {
                Some(8123) => Some(9000), // HTTP ClickHouse -> Insecure native
                Some(port) => Some(port), // Keep existing port
                None => Some(9000),       // Default insecure port
            };
            if let Some(port) = new_port {
                url.set_port(Some(port)).unwrap();
            }
        }
        "https" => {
            // Convert HTTPS ports to ClickHouse native ports
            let current_port = url.port();
            let new_port = match current_port {
                Some(8443) => Some(9440), // HTTPS ClickHouse -> Secure native
                Some(port) => Some(port), // Keep existing port
                None => Some(9440),       // Default secure port
            };
            if let Some(port) = new_port {
                url.set_port(Some(port)).unwrap();
            }
        }
        "clickhouse" => {
            // For clickhouse:// URLs, determine the appropriate port
            let current_port = url.port();
            let host = url.host_str().unwrap_or("localhost");

            // Cloud services should use secure port by default
            let is_cloud = !matches!(host, "localhost" | "127.0.0.1");
            let new_port = match current_port {
                Some(port) => Some(port), // Keep explicit port
                None => {
                    // Default port based on whether it's cloud or local
                    if is_cloud {
                        Some(9440)
                    } else {
                        Some(9000)
                    }
                }
            };

            if let Some(port) = new_port {
                url.set_port(Some(port)).unwrap();
            }
        }
        _ => {
            debug!(
                "URL scheme is not HTTP(S) or clickhouse://, returning unchanged: {}",
                url.scheme()
            );
            return Ok(url);
        }
    }

    // Move database from query parameter back to path if it exists (inverse operation)
    if let Some(database) = url
        .query_pairs()
        .find(|(k, _)| k == "database")
        .map(|(_, v)| v.to_string())
    {
        if !database.is_empty() {
            url.set_path(&format!("/{}", database));
            // Remove database from query parameters
            let new_query = url
                .query_pairs()
                .filter(|(k, _)| k != "database")
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            url.set_query(if new_query.is_empty() {
                None
            } else {
                Some(&new_query)
            });
        }
    }

    debug!(
        "Converted URL to native ClickHouse: {}://{}:{}{}",
        url.scheme(),
        url.host_str().unwrap_or("localhost"),
        url.port().unwrap_or(9000),
        url.path()
    );

    Ok(url)
}
