use base64::prelude::*;
use http_body_util::BodyExt;
use http_body_util::Full;

use hyper::body::Bytes;
use hyper::{Request, Response, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::{connect::HttpConnector, Client};

use super::config::ClickHouseConfig;
use super::model::ClickHouseRecord;

use log::error;

pub struct ClickHouseClient {
    client: Client<HttpConnector, Full<Bytes>>,
    ssl_client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    config: ClickHouseConfig,
}

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

    async fn request(
        &self,
        req: Request<Full<Bytes>>,
    ) -> Result<Response<hyper::body::Incoming>, hyper_util::client::legacy::Error> {
        if self.config.use_ssl {
            self.ssl_client.request(req).await
        } else {
            self.client.request(req).await
        }
    }

    pub async fn ping(&mut self) -> anyhow::Result<()> {
        let empty_body = Bytes::new();

        let req = Request::builder()
            .method("GET")
            .uri("/ping")
            .body(Full::new(empty_body))?;

        let res: Response<hyper::body::Incoming> = self.request(req).await?;

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
                        Some(value) => format!("{}", value),
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

        let query: String = query_param(&insert_query)?;
        let uri = self.uri(format!("/?{}", query))?;

        let body = Self::build_body(columns, records);
        let bytes = Bytes::from(body);

        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("Host", self.host())
            .header("Authorization", self.auth_header())
            .header("Content-Length", bytes.len())
            .body(Full::new(bytes))?;

        let res = self.request(req).await?;

        let status = res.status();

        if status != 200 {
            let body = res.collect().await?.to_bytes().to_vec();
            let body_str = String::from_utf8(body)?;

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
    let params = &[("query", Some(query))];
    let encoded = serde_urlencoded::to_string(params)?;

    Ok(encoded)
}
