use std::{sync::{Arc, Mutex}, collections::HashSet, path::{PathBuf, self}, io::Error};

use notify::{RecommendedWatcher, Config, RecursiveMode, Watcher, event::ModifyKind};

use crate::{framework::directories::get_app_directory, cli::user_messages::show_message};

use super::{CommandTerminal, user_messages::{MessageType, Message}};

fn process_event(project_dir: PathBuf, event: notify::Event, route_table:  Arc<Mutex<HashSet<PathBuf>>>) {
    match event.kind {
        notify::EventKind::Create(_) => {
            let route = event.paths[0].clone();
            let clean_route = route.strip_prefix(project_dir).unwrap().to_path_buf();
            let mut route_table = route_table.lock().unwrap();
            route_table.insert(clean_route);
        },
        notify::EventKind::Modify(mk) => {
            match mk {
                ModifyKind::Name(_) => {
                    // If the file is renamed, remove the old route and add the new one
                    let route = event.paths[0].clone();
                    let clean_route = route.strip_prefix(project_dir).unwrap().to_path_buf();
                    let mut route_table = route_table.lock().unwrap();

                    // remove the file from the routes if they don't exist in the file directory
                    if route.exists() {
                        route_table.insert(clean_route);
                    } else {
                        route_table.remove(&clean_route);
                    };

                }
                _ => {}
            }
        },   
        notify::EventKind::Remove(_) => {},
        _ => {}         
    }
}

fn watch(path: PathBuf, route_table: &Arc<Mutex<HashSet<PathBuf>>> ) -> notify::Result<()> {

    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                process_event(path.clone(), event.clone(), Arc::clone(&route_table));
            },
            Err(error) => println!("Error: {error:?}"),
        }
    }
    Ok(())
}

pub fn start_file_watcher(term: &mut CommandTerminal, route_table:  Arc<Mutex<HashSet<PathBuf>>>) -> Result<(), Error> {

    let path = get_app_directory(term)?;

    show_message(term, MessageType::Info, {
        Message {
            action: "Watching",
            details: &format!("{:?}", path.display()),
        }
    });

    tokio::spawn( async move {
        if let Err(error) = watch(path, &route_table) {
            println!("Error: {error:?}");
        }
    });
    
    Ok(())
}