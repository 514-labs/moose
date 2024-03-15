use base64::prelude::*;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::{connect::HttpConnector, Client};

use super::config::ClickhouseConfig;

struct ClickhouseRecord {
    columns: Vec<String>,
    values: Vec<String>,
}

struct ClickhouseClient {
    client: Client<HttpConnector, Full<Bytes>>,
    ssl_client: Client<HttpsConnector<HttpConnector>, Full<Bytes>>,
    config: ClickhouseConfig,
}

// TODO - handle different types of values for the insert
// TODO - add clikhouse container for tests inside github actions
// TODO - make sure we are safe with columns / values alignment
// TODO - implement batch inserts
// TODO - investigate if we need to change basic auth
impl ClickhouseClient {
    pub async fn new(clickhouse_config: ClickhouseConfig) -> anyhow::Result<Self> {
        let client_builder = Client::builder(hyper_util::rt::TokioExecutor::new());

        let https = HttpsConnector::new();
        let http = HttpConnector::new();

        Ok(Self {
            client: client_builder.build(http),
            ssl_client: client_builder.build(https),
            config: clickhouse_config,
        })
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

    pub async fn insert(
        &mut self,
        table_name: &str,
        record: ClickhouseRecord,
    ) -> anyhow::Result<()> {
        let insert_query = format!(
            "INSERT INTO {}.{} ({}) VALUES",
            self.config.db_name,
            table_name,
            record.columns.join(","),
        );

        let query: String = query_param(&insert_query)?;
        let uri = self.uri(format!("/?{}", query))?;

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

        let res = self.request(req).await?;

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

    // let clickhouse_config = ClickhouseConfig {
    //     user: "default".to_string(),
    //     password: "password".to_string(),
    //     host: "swtrnxdyro.us-central1.gcp.clickhouse.cloud".to_string(),
    //     use_ssl: true,
    //     postgres_port: 5432,
    //     kafka_port: 9092,
    //     host_port: 8443,
    //     db_name: "default".to_string(),
    // };

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
