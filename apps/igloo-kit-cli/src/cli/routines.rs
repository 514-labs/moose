use std::path::Path;
use std::{io::Error, path::PathBuf};

use crate::{infrastructure, framework};

use super::{CommandTerminal, user_messages::show_message, MessageType, Message};

use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Config};





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

fn watch<P: AsRef<Path>>(path: P) -> notify::Result<()> {

    let (tx, rx) = std::sync::mpsc::channel();

    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => println!("Change: {event:?}"),
            Err(error) => println!("Error: {error:?}"),
        }
    }

    Ok(())
}


pub fn start_file_watcher() -> Result<(), Error> {

    let path = "/Users/timdelisle/Dev/igloo-stack/apps/igloo-kit-cli/app";

    println!("Watching {path}");

    tokio::spawn( async move {
        if let Err(error) = watch(path) {
            println!("Error: {error:?}");
        }
    });
    
    Ok(())
}




// TODO Figure out how to stop the web server
pub async fn start_webserver(term: &mut CommandTerminal) {
    let addr = SocketAddr::from(([127,0,0,1], 4000));


    async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
        Ok(Response::new("Hello, World".into()))
    }


    show_message( term, MessageType::Info, Message {
        action: "starting",
        details: " server on port 4000",
    });

    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
    