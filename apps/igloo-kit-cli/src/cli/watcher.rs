use std::{sync::{Arc, RwLock}, collections::HashMap, path::PathBuf, io::{Error, ErrorKind}};

use notify::{RecommendedWatcher, Config, RecursiveMode, Watcher, event::ModifyKind};
use tokio::sync::Mutex;

use crate::{framework::{directories::get_app_directory, schema::{parse_schema_file, TableOps, MatViewOps}}, cli::display::show_message, infrastructure::{stream, olap::{self, clickhouse::{ConfiguredClient, mapper, ClickhouseTable, config::ClickhouseConfig, ClickhouseView}}}};

use super::{CommandTerminal, display::{MessageType, Message}};

fn dataframe_path_to_ingest_route(project_dir: PathBuf, path: PathBuf, table_name: String) -> PathBuf {
    let dataframe_path = project_dir.join("dataframes");
    let mut route = path.strip_prefix(dataframe_path).unwrap().to_path_buf();

    route.set_file_name(table_name);

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
                        remove_table_and_topics_from_dataframe_route(&route, &mut route_table, configured_client).await
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
    pub original_file_path: PathBuf,
    pub table_name: String,
    pub view_name: Option<String>,
}

async fn create_table_and_topics_from_dataframe_route(route: &PathBuf, project_dir: PathBuf, route_table: &mut tokio::sync::MutexGuard<'_, HashMap::<PathBuf, RouteMeta>>, configured_client: &ConfiguredClient) -> Result<(), Error> {
    if let Some(ext) = route.extension() {
            if ext == "prisma" && route.as_path().to_str().unwrap().contains("dataframes")  {
                let tables = parse_schema_file::<ClickhouseTable>(route.clone(), mapper::std_table_to_clickhouse_table)
                    .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to parse schema file. Error {}", e)))?;
    
                for table in tables {
                    let ingest_route = dataframe_path_to_ingest_route(project_dir.clone(), route.clone(), table.name.clone());
                    
                    stream::redpanda::create_topic_from_name(table.name.clone());
                    let table_query = table.create_table_query()
                        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to get clickhouse query: {:?}", e)))?;
                    
                    olap::clickhouse::run_query(table_query, configured_client).await
                        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to create table in clickhouse: {}", e)))?;

                    let view = ClickhouseView::new(
                        table.db_name.clone(), 
                        format!("{}_view", table.name), 
                        table.clone());
                    let view_query = view.create_materialized_view_query()
                        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to get clickhouse query: {:?}", e)))?;

                    olap::clickhouse::run_query(view_query, configured_client).await
                        .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to create table in clickhouse: {}", e)))?;

                    route_table.insert(ingest_route, RouteMeta { original_file_path: route.clone(), table_name: table.name.clone(), view_name: Some(view.name.clone()) });
                };
            }
        } else {
            println!("No primsa extension found. Likely created unsupported file type")
        }
    Ok(())
}

async fn remove_table_and_topics_from_dataframe_route(route: &PathBuf, route_table: &mut tokio::sync::MutexGuard<'_, HashMap::<PathBuf, RouteMeta>>, configured_client: &ConfiguredClient) -> Result<(), Error> {
    //need to get the path of the file, scan the route table and remove all the files that need to be deleted. 
    // This doesn't have to be as fast as the scanning for routes in the web server so we're ok with the scan here.
    for (k, meta) in route_table.clone().into_iter() {
        if meta.original_file_path == route.clone() {
            stream::redpanda::delete_topic(meta.table_name.clone());
            
            olap::clickhouse::delete_table_or_view(meta.table_name, configured_client).await
                .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to create table in clickhouse: {}", e)))?;

            if let Some(view_name) = meta.view_name {
                olap::clickhouse::delete_table_or_view(view_name, configured_client).await
                    .map_err(|e| Error::new(ErrorKind::Other, format!("Failed to create table in clickhouse: {}", e)))?;
            }

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

pub fn start_file_watcher(term: Arc<RwLock<CommandTerminal>>, route_table:  Arc<Mutex<HashMap::<PathBuf, RouteMeta>>>, clickhouse_config: ClickhouseConfig) -> Result<(), Error> {

    let path = get_app_directory()?;

    show_message(term, MessageType::Info, {
        Message {
            action: "Watching".to_string(),
            details: format!("{:?}", path.display()),
        }
    });

    tokio::spawn( async move {
        // Need to spin up client in thread to ensure it lives long enough
        let db_client = olap::clickhouse::create_client(clickhouse_config.clone());

        if let Err(error) = watch(path, Arc::clone(&route_table), &db_client).await {
            println!("Error: {error:?}");
        }
    });
    
    Ok(())
}