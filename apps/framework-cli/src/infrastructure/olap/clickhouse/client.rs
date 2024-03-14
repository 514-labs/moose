use std::error::Error as StdError;

use super::config::ClickhouseConfig;
use base64::prelude::*;
use http_body_util::{Empty, Full};
use hyper::body::{Body, Bytes};
use hyper::client::conn::http1::SendRequest;
use hyper::{Request, Uri};
use hyper_util::rt::TokioIo;
use log::error;
use std::sync::Arc;
use tokio::net::TcpStream;

struct ClickhouseRecord {
    columns: Vec<String>,
    values: Vec<String>,
}

struct ClickhouseClient {
    client: SendRequest<Full<Bytes>>,
    config: ClickhouseConfig,
}

// TODO - Implement SSL
// TODO - handle different types of values for the insert
// TODO - make sure we are safe with columns / values alignment
// TODO - implement batch inserts
// TODO - investigate if we need to change basic auth
impl ClickhouseClient {
    pub async fn new(clickhouse_config: ClickhouseConfig) -> anyhow::Result<Self> {
        // let url = clickhouse_config. .parse::<hyper::Uri>()?;

        // let scheme = if clickhouse_config.use_ssl {
        //     "https"
        // } else {
        //     "http"
        // };

        let authority = if clickhouse_config.host_port == 443 || clickhouse_config.host_port == 80 {
            clickhouse_config.host.clone()
        } else {
            format!("{}:{}", clickhouse_config.host, clickhouse_config.host_port)
        };

        // Open a TCP connection to the remote host
        let stream = TcpStream::connect(authority).await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Create the Hyper client
        let (sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        // Spawn a task to poll the connection, driving the HTTP state
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                error!("Connection failed: {:?}", err);
            }
        });

        Ok(Self {
            client: sender,
            config: clickhouse_config,
        })
    }

    async fn ping(&mut self) -> anyhow::Result<()> {
        let empty_body = Bytes::new();

        let req = Request::builder()
            .method("GET")
            .uri("/ping")
            .body(Full::new(empty_body))?;

        let res = self.client.send_request(req).await.unwrap();

        assert_eq!(res.status(), 200);
        Ok(())
    }

    fn auth_header(&self) -> String {
        // TODO properly encode basic auth
        let username_and_password = format!("{}:{}", self.config.user, self.config.password);
        let encoded = BASE64_STANDARD.encode(username_and_password);
        format!("Basic {}", encoded)
    }

    fn host(&self) -> String {
        format!("{}:{}", self.config.host, self.config.host_port)
    }

    async fn insert(&mut self, table_name: &str, record: ClickhouseRecord) -> anyhow::Result<()> {
        let insert_query = format!(
            "INSERT INTO {}.{} ({}) VALUES",
            self.config.db_name,
            table_name,
            record.columns.join(","),
        );

        let query: String = query_param(&insert_query)?;
        let uri = format!("/?{}", query);

        let body = record
            .values
            .iter()
            .map(|value| format!("('{}')", value))
            .collect::<Vec<String>>()
            .join(",");

        let bytes = Bytes::from(body);

        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Host", self.host())
            .header("Authorization", self.auth_header())
            .header("Content-Length", bytes.len())
            .body(Full::new(bytes))?;

        let res = self.client.send_request(req).await.unwrap();

        assert_eq!(res.status(), 200);

        Ok(())
    }
}

fn query_param(query: &str) -> anyhow::Result<String> {
    let params = &[("query", Some(query))];
    let encoded = serde_urlencoded::to_string(params)?;

    Ok(encoded)
}

#[tokio::test]
async fn test_ping() {
    let clickhouse_config = ClickhouseConfig {
        user: "panda".to_string(),
        password: "pandapass".to_string(),
        host: "localhost".to_string(),
        use_ssl: false,
        postgres_port: 5432,
        kafka_port: 9092,
        host_port: 18123,
        db_name: "local".to_string(),
    };

    let mut client = ClickhouseClient::new(clickhouse_config).await.unwrap();

    client.ping().await.unwrap();
}

#[tokio::test]
async fn test_insert() {
    let clickhouse_config = ClickhouseConfig {
        user: "panda".to_string(),
        password: "pandapass".to_string(),
        host: "localhost".to_string(),
        use_ssl: false,
        postgres_port: 5432,
        kafka_port: 9092,
        host_port: 18123,
        db_name: "local".to_string(),
    };

    let mut client = ClickhouseClient::new(clickhouse_config).await.unwrap();

    client
        .insert(
            "test_table",
            ClickhouseRecord {
                columns: vec!["name".to_string()],
                values: vec!["panda".to_string()],
            },
        )
        .await
        .unwrap();
}
