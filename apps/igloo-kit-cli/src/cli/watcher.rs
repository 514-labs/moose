use std::{sync::Arc, collections::HashMap, path::PathBuf, io::{Error, ErrorKind}};

use notify::{RecommendedWatcher, Config, RecursiveMode, Watcher, event::ModifyKind};
use tokio::sync::Mutex;

use crate::{framework::{directories::get_app_directory, schema::{parse_schema_file, OpsTable}}, cli::user_messages::show_message, infrastructure::{stream, db::{self, clickhouse::{ConfiguredClient, ClickhouseConfig}}}};

use super::{CommandTerminal, user_messages::{MessageType, Message}};

fn route_to_topic_name(route: PathBuf) -> String {
    let route = route.to_str().unwrap().to_string();
    let route = route.replace("/", ".");
    route
}

fn dataframe_path_to_ingest_route(project_dir: PathBuf, path: PathBuf) -> PathBuf {
    let dataframe_path = project_dir.join("dataframes");
    let route = path.strip_prefix(dataframe_path).unwrap().to_path_buf();
    PathBuf::from("ingest").join(route)
}

async fn process_event(project_dir: PathBuf, event: notify::Event, route_table:  Arc<Mutex<HashMap::<PathBuf, RouteMeta>>>, configured_client: &ConfiguredClient) -> Result<(), Error> {
    let route = event.paths[0].clone();
    let mut route_table = route_table.lock().await;

    match event.kind {
        notify::EventKind::Create(_) => {
            // Only create tables and topics from prisma files in the dataframes directory
            create_table_and_topics_from_dataframe_route(&route, project_dir, &mut route_table, configured_client).await
        },
        notify::EventKind::Modify(mk) => {
            match mk {
                ModifyKind::Name(_) => {
                    // remove the file from the routes if they don't exist in the file directory
                    if route.exists() {
                        create_table_and_topics_from_dataframe_route(&route, project_dir, &mut route_table, configured_client).await
                            
                    } else {
                        remove_table_and_topics_from_dataframe_route(&route, project_dir, &mut route_table, configured_client).await
                    }
                }
                _ => {Ok(())}
            }
        },   
        notify::EventKind::Remove(_) => {Ok(())},
        _ => {Ok(())}         
    }
}

#[derive(Debug, Clone)]
pub struct RouteMeta {
    original_file_path: PathBuf,
    table_name: String,
}

async fn create_table_and_topics_from_dataframe_route(route: &PathBuf, project_dir: PathBuf, route_table: &mut tokio::sync::MutexGuard<'_, HashMap::<PathBuf, RouteMeta>>, configured_client: &ConfiguredClient) -> Result<(), Error> {
    if let Some(ext) = route.extension() {
            if ext == "prisma" && route.as_path().to_str().unwrap().contains("dataframes")  {
                let tables = parse_schema_file(route.clone())
                    .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to parse schema file. Error {}", e)))?;
    
                for table in tables {
                    let ingest_route = dataframe_path_to_ingest_route(project_dir.clone(), route.clone());
                    route_table.insert(ingest_route, RouteMeta { original_file_path: route.clone(), table_name: table.name.clone() });
                    stream::redpanda::create_topic_from_name(table.name.clone());
                    let query = table.create_table_query()
                        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to get clickhouse query: {:?}", e)))?;
                    db::clickhouse::run_query(query, configured_client).await
                        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to create table in clickhouse: {}", e)))?;
                };
            }
        } else {
            println!("No primsa extension found. Likely created unsupported file type")
        }
    Ok(())
}

async fn remove_table_and_topics_from_dataframe_route(route: &PathBuf, project_dir: PathBuf, route_table: &mut tokio::sync::MutexGuard<'_, HashMap::<PathBuf, RouteMeta>>, configured_client: &ConfiguredClient) -> Result<(), Error> {
    //need to get the path of the file, scan the route table and remove all the files that need to be deleted. 
    // This doesn't have to be as fast as the scanning for routes in the web server so we're ok with the scan here.
    for (k, meta) in route_table.clone().into_iter() {
        if meta.original_file_path == route.clone() {
            let ingest_route = k.clone();
            stream::redpanda::delete_topic(ingest_route.to_str().unwrap().to_string());
            
            db::clickhouse::delete_table(meta.table_name, configured_client).await
                .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to create table in clickhouse: {}", e)))?;

            route_table.remove(&k);
        }
    }
    Ok(())
}

async fn watch(path: PathBuf, route_table: Arc<Mutex<HashMap::<PathBuf, RouteMeta>>>, configured_client: &ConfiguredClient ) -> Result<(), Error> {

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default()).map_err(
        |e| Error::new(ErrorKind::Other, format!("Failed to create file watcher: {}", e))
    )?;

    watcher.watch(path.as_ref(), RecursiveMode::Recursive).map_err(
        |e| Error::new(ErrorKind::Other, format!("Failed to watch file: {}", e))
    )?;

    for res in rx {
        match res {
            Ok(event) => {
                process_event(path.clone(), event.clone(), Arc::clone(&route_table), configured_client).await.map_err(
                    |e| Error::new(ErrorKind::Other, format!("clickhouse error has occured: {}", e))
                )?;
            },
            Err(error) => return Err(Error::new(ErrorKind::Other, format!("File watcher event caused a failure: {}", error)))
        }
        println!("{:?}", route_table)
    }
    Ok(())
}

pub fn start_file_watcher(term: &mut CommandTerminal, route_table:  Arc<Mutex<HashMap::<PathBuf, RouteMeta>>>, clickhouse_config: ClickhouseConfig) -> Result<(), Error> {

    let path = get_app_directory(term)?;

    show_message(term, MessageType::Info, {
        Message {
            action: "Watching",
            details: &format!("{:?}", path.display()),
        }
    });

    tokio::spawn( async move {
        // Need to spin up client in thread to ensure it lives long enough
        let db_client = db::clickhouse::create_client(clickhouse_config.clone());

        if let Err(error) = watch(path, Arc::clone(&route_table), &db_client).await {
            println!("Error: {error:?}");
        }
    });
    
    Ok(())
}