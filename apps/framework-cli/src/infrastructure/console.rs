use std::collections::{HashMap, HashSet};
use std::str;

use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::Method;
use hyper::Request;
use hyper_util::rt::TokioIo;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use tokio::net::TcpStream;

use crate::framework::controller::{
    schema_file_path_to_ingest_route, FrameworkObjectVersions, SchemaVersion,
};
use crate::framework::data_model::schema::DataModel;
use crate::infrastructure::olap;
use crate::infrastructure::olap::clickhouse::model::ClickHouseSystemTable;
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
    olap::clickhouse::check_ready(configured_db_client)
        .await
        .unwrap();
    let tables = olap::clickhouse::fetch_all_tables(configured_db_client)
        .await
        .unwrap()
        .into_iter()
        .map(|table| (table.name.clone(), table))
        .collect::<HashMap<_, _>>();
    let topics = HashSet::<String>::from_iter(
        redpanda::fetch_topics(&configured_producer.config)
            .await
            .unwrap()
            .into_iter(),
    );

    let flows = project.get_flows();

    let current_version = Version {
        models: serialize_version(
            &framework_object_versions.current_models,
            &framework_object_versions.current_version,
            &tables,
            &topics,
            &flows,
        ),
    };

    let past_versions = framework_object_versions
        .previous_version_models
        .iter()
        .map(|(version, schema_version)| {
            (
                version.clone(),
                Version {
                    models: serialize_version(schema_version, version, &tables, &topics, &flows),
                },
            )
        })
        .collect::<HashMap<_, _>>();

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
            "current": current_version,
            "past": past_versions,
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
struct RouteInfo {
    pub route_path: String,
    pub file_path: String,
    pub table_name: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ModelInfra {
    model: DataModel,
    ingestion_point: RouteInfo,
    table: Option<ClickHouseSystemTable>,
    // queue is None if it is the same as previous version
    // then we don't need to spin up a queue and a table
    queue: Option<String>,
    flows: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct Version {
    models: Vec<ModelInfra>,
}

impl RouteInfo {
    pub fn new(route_path: String, file_path: String, table_name: Option<String>) -> Self {
        Self {
            route_path,
            file_path,
            table_name,
        }
    }
}

fn serialize_version(
    schema_version: &SchemaVersion,
    version: &str,
    tables: &HashMap<String, ClickHouseSystemTable>,
    topics: &HashSet<String>,
    flows: &HashMap<String, Vec<String>>,
) -> Vec<ModelInfra> {
    schema_version
        .models
        .values()
        .map(|fo| {
            let route_path = schema_file_path_to_ingest_route(
                &schema_version.base_path,
                &fo.original_file_path,
                fo.data_model.name.clone(),
                version,
            )
            .to_string_lossy()
            .to_string();

            let ingestion_point = RouteInfo::new(
                route_path,
                fo.original_file_path.to_str().unwrap().to_string(),
                fo.table.clone().map(|table| table.name),
            );
            ModelInfra {
                model: fo.data_model.clone(),
                ingestion_point,
                table: fo.table.clone().map(|table| match tables.get(&table.name) {
                    Some(table) => table.clone(),
                    None => {
                        warn!("Table not found: {}", table.name);
                        ClickHouseSystemTable {
                            uuid: "".to_string(),
                            database: "".to_string(),
                            name: "table_not_found".to_string(),
                            dependencies_table: vec![],
                            engine: "".to_string(),
                        }
                    }
                }),
                queue: if topics.contains(&fo.topic) {
                    Some(fo.topic.clone())
                } else {
                    None
                },
                flows: flows.get(&fo.data_model.name).unwrap_or(&vec![]).clone(),
            }
        })
        .collect()
}
