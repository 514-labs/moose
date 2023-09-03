use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use std::{io::Error, path::PathBuf};

use crate::{infrastructure, framework};

use super::{CommandTerminal, user_messages::show_message, MessageType, Message};

use std::convert::Infallible;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Config};

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use std::sync::Mutex;




pub fn start_containers(term: &mut CommandTerminal) -> Result<(), Error> {
    show_message( term, MessageType::Info, Message {
        action: "Running",
        details: "infrastructure spin up",
    });
    
    infrastructure::spin_up(term)?;
    Ok(())
}

pub fn initialize_project(term: &mut CommandTerminal) -> Result<(), Error> {
    let igloo_dir = framework::directories::create_top_level_temp_dir(term)?;
    match framework::directories::create_project_directories(term) {
        Ok(_) => {
            show_message( term, MessageType::Success, Message {
                action: "Finished",
                details: "initializing project directory",
            });
        },
        Err(err) => {
            show_message( term, MessageType::Error, Message {
                action: "Failed",
                details: "to create project directories",
            });
            return Err(err)
        }
    };
    infrastructure::init(term, &igloo_dir)?;
    Ok(())
}

pub fn clean_project(term: &mut CommandTerminal, igloo_dir: &PathBuf) -> Result<(), Error> {
    show_message( term, MessageType::Info, Message {
        action: "Cleaning",
        details: "project directory",
    });
    infrastructure::clean(term, igloo_dir)?;
    show_message(
        term,
        MessageType::Success,
        Message {
            action: "Finished",
            details: "cleaning project directory",
        },
    );
    Ok(())
}

pub fn stop_containers(term: &mut CommandTerminal) -> Result<(), Error> {
    show_message( term, MessageType::Info, Message {
        action: "Stopping",
        details: "local infrastructure",
    });
    infrastructure::spin_down(term)?;
    Ok(())
}

fn watch<P: AsRef<Path>>(path: P, route_table: &Arc<Mutex<HashSet<String>>> ) -> notify::Result<()> {

    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                match event.kind {
                    notify::EventKind::Create(_) => {
                        let route = event.paths[0].file_name().unwrap().to_str().unwrap().to_string();
                        let mut route_table = route_table.lock().unwrap();
                        route_table.insert("/".to_owned() + &route);
                        println!("Route table: {:#?}", route_table)
                    },
                    notify::EventKind::Remove(_) => {
                        let route = event.paths[0].file_name().unwrap().to_str().unwrap().to_string();
                        let mut route_table = route_table.lock().unwrap();
                        let path = "/".to_owned() + &route;
                        route_table.remove(&path);
                        println!("Route table: {:#?}", route_table)
                    },
                    _ => {}
                }
            },
            Err(error) => println!("Error: {error:?}"),
        }
    }

    Ok(())
}

pub async fn start_development_mode(term: &mut CommandTerminal) -> Result<(), Error> {
    show_message( term, MessageType::Success, Message {
        action: "Starting",
        details: "development mode...",
    });

    let route_table = Arc::new(Mutex::new(HashSet::<String>::new()));

    start_file_watcher(Arc::clone(&route_table))?;
    start_webserver(term, Arc::clone(&route_table)).await;
    Ok(())
}

pub fn start_file_watcher(route_table:  Arc<Mutex<HashSet<String>>>) -> Result<(), Error> {

    let path = PathBuf::from("/Users/timdelisle/Dev/igloo-stack/apps/igloo-kit-cli/app");

    println!("Watching {:?}", path.display());

    tokio::spawn( async move {
        if let Err(error) = watch(path, &route_table) {
            println!("Error: {error:?}");
        }
    });
    
    Ok(())
}

// TODO Figure out how to stop the web server
pub async fn start_webserver(term: &mut CommandTerminal, route_table: Arc<Mutex<HashSet<String>>>) {
    
    let addr = ([127, 0, 0, 1], 4000).into();


    async fn handler(req: Request<Body>, route_table: Arc<Mutex<HashSet<String>>>) -> Result<Response<String>, Infallible> {
        let route = req.uri().path().to_string();

        println!("Route: {}", route);
        // Check if route is in the route table
        if route_table.lock().unwrap().contains(&route) {
            return Ok(Response::builder()
            .status(StatusCode::FOUND)
            .body("FOUND".to_string())
            .unwrap())
            
        }
        
        // If not, return a 404
        Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("NOTFOUND".to_string())
                .unwrap())
    }


    show_message( term, MessageType::Info, Message {
        action: "starting",
        details: " server on port 4000",
    });

    let main_service = make_service_fn(move |_| {
        let route_table = route_table.clone();

        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                handler(req, route_table.clone())
            }))
        }

    });

    let server = Server::bind(&addr).serve(main_service);

    // Run this server for... forever!
    if let Err(e) = server.await {
        show_message(term, MessageType::Error, Message {
            action: "Error",
            details: e.to_string().as_str(),
        });
    }
}
    