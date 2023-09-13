use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;
use hyper::StatusCode;
use hyper::service::service_fn;
use tokio::sync::Mutex;
use std::convert::Infallible;
use std::path::PathBuf;
use hyper::service::make_service_fn;
use super::Message;
use super::MessageType;
use super::user_messages::show_message;
use std::collections::HashSet;
use std::sync::Arc;
use super::CommandTerminal;


async fn handler(req: Request<Body>, route_table: Arc<Mutex<HashSet<PathBuf>>>) -> Result<Response<String>, Infallible> {
    let route_prefix = PathBuf::from("/");
    let route = PathBuf::from(req.uri().path()).strip_prefix(route_prefix).unwrap().to_path_buf();

    // Check if route is in the route table
    if route_table.lock().await.contains(&route) {
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

// TODO Figure out how to stop the web server
pub async fn start_webserver(term: &mut CommandTerminal, route_table: Arc<Mutex<HashSet<PathBuf>>>) {

    let addr = ([127, 0, 0, 1], 4000).into();

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