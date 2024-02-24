use std::str;

use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Method;
use hyper::Request;
use hyper_util::rt::TokioIo;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use tokio::net::TcpStream;

use crate::framework::controller::{schema_file_path_to_ingest_route, FrameworkObjectVersions};
use crate::framework::schema::DataModel;
use crate::infrastructure::olap;
use crate::infrastructure::olap::clickhouse::ConfiguredDBClient;
use crate::infrastructure::stream::redpanda;
use crate::infrastructure::stream::redpanda::ConfiguredProducer;
use crate::project::Project;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConsoleConfig {
    pub host_port: u16, // ex. 18123
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self { host_port: 3001 }
    }
}

pub async fn post_current_state_to_console(
    project: Arc<Project>,
    configured_db_client: &ConfiguredDBClient,
    configured_producer: &ConfiguredProducer,
    framework_object_versions: &FrameworkObjectVersions,
) -> Result<(), anyhow::Error> {
    let models: Vec<DataModel> = framework_object_versions
        .current_models
        .models
        .values()
        .map(|fo| fo.data_model.clone())
        .collect();

    olap::clickhouse::check_ready(configured_db_client)
        .await
        .unwrap();
    let tables = olap::clickhouse::fetch_all_tables(configured_db_client)
        .await
        .unwrap();
    let topics = redpanda::fetch_topics(&configured_producer.config)
        .await
        .unwrap();

    // TODO: old versions of the models are not being sent to the console
    let routes_table: Vec<RouteInfo> = framework_object_versions
        .current_models
        .models
        .values()
        .map(|fo| {
            let route_path = schema_file_path_to_ingest_route(
                &framework_object_versions.current_models.base_path,
                &fo.original_file_path,
                fo.data_model.name.clone(),
                &framework_object_versions.current_version,
            )
            .to_string_lossy()
            .to_string();

            RouteInfo::new(
                route_path,
                fo.original_file_path.to_str().unwrap().to_string(),
                fo.table.name.clone(),
                Some(fo.table.view_name()),
            )
        })
        .collect();

    // TODO this should be configurable
    let url = format!(
        "http://localhost:{}/api/console",
        project.console_config.host_port
    )
    .parse::<hyper::Uri>()?;

    let host = url.host().expect("uri has no host");
    let port = url.port_u16().unwrap();
    let address = format!("{}:{}", host, port);

    debug!("Connecting to moose console at: {}", address);

    let stream = TcpStream::connect(address).await?;
    let io = TokioIo::new(stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            debug!("Connection failed: {:?}", err);
        }
    });

    let body = Bytes::from(
        json!({
            "project": json!(*project),
            "models": models,
            "tables": tables,
            "queues": topics,
            "ingestionPoints": routes_table
        })
        .to_string(),
    );

    let req = Request::builder()
        .uri(url.path())
        .method(Method::POST)
        .header("Host", "localhost:3001")
        .header("Content-Type", "application/json")
        .header("Content-Length", body.len())
        .header("Accept", "*/*")
        .header("User-Agent", "Hyper.rs")
        .body(Full::new(body))?;

    debug!("Sending CLI data to moose console: {:?}", req);

    let res = sender.send_request(req).await?;

    debug!("Response from Moose Console: {:?}", res);

    let body = res.collect().await.unwrap().to_bytes().to_vec();
    debug!(
        "Response from Moose Console: {:?}",
        str::from_utf8(&body).unwrap()
    );

    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RouteInfo {
    pub route_path: String,
    pub file_path: String,
    pub table_name: String,
    pub view_name: Option<String>,
}

impl RouteInfo {
    pub fn new(
        route_path: String,
        file_path: String,
        table_name: String,
        view_name: Option<String>,
    ) -> Self {
        Self {
            route_path,
            file_path,
            table_name,
            view_name,
        }
    }
}
