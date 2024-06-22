use base64::prelude::*;
use http_body_util::BodyExt;
use http_body_util::Full;

use async_recursion::async_recursion;
use hyper::body::Bytes;
use hyper::{Request, Response, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use log::debug;
use tokio::time::{sleep, Duration};

use super::config::ClickHouseConfig;
use super::model::ClickHouseRecord;

use log::error;

pub struct ClickHouseClient {
    client: Client<HttpConnector, Full<Bytes>>,
    ssl_client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    config: ClickHouseConfig,
}

// Considering Clickhouse could take 30s to wake up, we need to have a backoff strategy
const BACKOFF_START_MILLIS: u64 = 1000;
const MAX_RETRIES: u8 = 10;
// Retries will be 1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s, 256s, 512s

// TODO - investigate if we need to change basic auth
impl ClickHouseClient {
    pub fn new(clickhouse_config: &ClickHouseConfig) -> anyhow::Result<Self> {
        let client_builder = Client::builder(hyper_util::rt::TokioExecutor::new());

        let https = HttpsConnector::new();
        let http = HttpConnector::new();

        Ok(Self {
            client: client_builder.build(http),
            ssl_client: client_builder.build(https),
            config: clickhouse_config.clone(),
        })
    }

    pub fn config(&self) -> &ClickHouseConfig {
        &self.config
    }

    #[async_recursion]
    async fn request(
        &self,
        req: Request<Full<Bytes>>,
        retries: u8,
        backoff_millis: u64,
    ) -> Result<Response<hyper::body::Incoming>, hyper_util::client::legacy::Error> {
        let res = if self.config.use_ssl {
            self.ssl_client.request(req.clone()).await
        } else {
            self.client.request(req.clone()).await
        };

        match res {
            Ok(res) => Ok(res),
            Err(e) => {
                println!("CLIENT: {}", e);
                if e.is_connect() {
                    if retries > 0 {
                        sleep(Duration::from_millis(backoff_millis)).await;
                        self.request(req, retries - 1, backoff_millis * 2).await
                    } else {
                        Err(e)
                    }
                } else {
                    Err(e)
                }
            }
        }
    }

    pub async fn ping(&mut self) -> anyhow::Result<()> {
        let empty_body = Bytes::new();

        let req = Request::builder()
            .method("GET")
            .uri("/ping")
            .body(Full::new(empty_body))?;

        let res: Response<hyper::body::Incoming> =
            self.request(req, MAX_RETRIES, BACKOFF_START_MILLIS).await?;

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

    fn uri(&self, path: String) -> anyhow::Result<Uri> {
        let scheme = if self.config.use_ssl { "https" } else { "http" };

        let uri = format!("{}://{}{}", scheme, self.host(), path);
        let parsed = uri.parse()?;

        Ok(parsed)
    }

    fn build_body(columns: &[String], records: &[ClickHouseRecord]) -> String {
        let value_list = records
            .iter()
            .map(|record| {
                columns
                    .iter()
                    .map(|column| match record.get(column) {
                        Some(value) => value.clickhouse_to_string(),
                        None => "NULL".to_string(),
                    })
                    .collect::<Vec<String>>()
                    .join(",")
            })
            .collect::<Vec<String>>()
            .join("),(");

        format!("({})", value_list)
    }

    pub async fn insert(
        &self,
        table_name: &str,
        columns: &[String],
        records: &[ClickHouseRecord],
    ) -> anyhow::Result<()> {
        // TODO - this could be optimized with RowBinary instead
        let insert_query = format!(
            "INSERT INTO {}.{} ({}) VALUES",
            self.config.db_name,
            table_name,
            columns.join(","),
        );

        debug!("Inserting into clickhouse: {}", insert_query);

        let query: String = query_param(&insert_query)?;
        let uri = self.uri(format!("/?{}", query))?;

        let body = Self::build_body(columns, records);
        println!("BODY {:?}", body);

        debug!("Inserting into clickhouse with values: {}", body);

        let bytes = Bytes::from(body);

        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Host", self.host())
            .header("Authorization", self.auth_header())
            .header("Content-Length", bytes.len())
            .body(Full::new(bytes))?;

        let res = self.request(req, MAX_RETRIES, BACKOFF_START_MILLIS).await?;

        let status = res.status();
        println!("REQ: {}", query);

        if status != 200 {
            let body = res.collect().await?.to_bytes().to_vec();
            let body_str = String::from_utf8(body)?;
            println!("HERE");
            println!("QUERY: {}", insert_query);
            error!(
                "Failed to insert into clickhouse: Res {} - {}",
                &status, body_str
            );

            Err(anyhow::anyhow!(
                "Failed to insert into clickhouse: {}",
                body_str
            ))
        } else {
            Ok(())
        }
    }
}

fn query_param(query: &str) -> anyhow::Result<String> {
    let params = &[("query", query), ("date_time_input_format", "best_effort")];
    let encoded = serde_urlencoded::to_string(params)?;

    Ok(encoded)
}
