use std::{sync::Arc, collections::HashSet, path::PathBuf, io::{Error, ErrorKind}};

use notify::{RecommendedWatcher, Config, RecursiveMode, Watcher, event::ModifyKind};
use tokio::sync::Mutex;

use crate::{framework::directories::get_app_directory, cli::user_messages::show_message, infrastructure::{stream, db::{self, clickhouse::{ConfiguredClient, ClickhouseConfig}}}};

use super::{CommandTerminal, user_messages::{MessageType, Message}};

fn route_to_topic_name(route: PathBuf) -> String {
    let route = route.to_str().unwrap().to_string();
    let route = route.replace("/", ".");
    route
}

async fn process_event(project_dir: PathBuf, event: notify::Event, route_table:  Arc<Mutex<HashSet<PathBuf>>>, configured_client: &ConfiguredClient) -> Result<(), clickhouse::error::Error> {
    let route = event.paths[0].clone();
    let clean_route = route.strip_prefix(project_dir).unwrap().to_path_buf();
    let table_name = clean_route.file_name().unwrap().to_str().unwrap().to_string();
    let topic_name = clean_route.file_name().unwrap().to_str().unwrap().to_string();
    let mut route_table = route_table.lock().await;

    match event.kind {
        notify::EventKind::Create(_) => {
            route_table.insert(clean_route.clone());
            
            stream::redpanda::create_topic_from_name(topic_name.clone());
            db::clickhouse::create_table(table_name, topic_name, configured_client).await
        },
        notify::EventKind::Modify(mk) => {
            match mk {
                ModifyKind::Name(_) => {
                    // remove the file from the routes if they don't exist in the file directory
                    if route.exists() {
                        route_table.insert(clean_route.clone());
                        stream::redpanda::create_topic_from_name(topic_name.clone());
                        db::clickhouse::create_table(table_name, topic_name, configured_client).await?
                    } else {
                        route_table.remove(&clean_route);
                        stream::redpanda::delete_topic_from_name(topic_name.clone());
                        db::clickhouse::delete_table(topic_name, configured_client).await?
                    };
                    Ok(())

                }
                _ => {Ok(())}
            }
        },   
        notify::EventKind::Remove(_) => {Ok(())},
        _ => {Ok(())}         
    }
}

async fn watch(path: PathBuf, route_table: Arc<Mutex<HashSet<PathBuf>>>, configured_client: &ConfiguredClient ) -> Result<(), Error> {

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
    }
    Ok(())
}

pub fn start_file_watcher(term: &mut CommandTerminal, route_table:  Arc<Mutex<HashSet<PathBuf>>>, clickhouse_config: ClickhouseConfig) -> Result<(), Error> {

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