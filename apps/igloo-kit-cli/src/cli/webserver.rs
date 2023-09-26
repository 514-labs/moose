use hyper::Body;
use hyper::Request;
use hyper::Response;
use hyper::Server;
use hyper::StatusCode;
use hyper::service::service_fn;
use rdkafka::producer::FutureRecord;
use rdkafka::util::Timeout;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::time::Duration;
use hyper::service::make_service_fn;
use crate::infrastructure::stream::redpanda;
use crate::infrastructure::stream::redpanda::ConfiguredProducer;
use crate::infrastructure::stream::redpanda::RedpandaConfig;

use super::Message;
use super::MessageType;
use super::display::show_message;
use super::watcher::RouteMeta;
use std::sync::Arc;
use super::CommandTerminal;


async fn handler(req: Request<Body>, route_table: Arc<Mutex<HashMap::<PathBuf, RouteMeta>>>, configured_producer: Arc<Mutex<ConfiguredProducer>>) -> Result<Response<String>, hyper::http::Error> {
    let route_prefix = PathBuf::from("/");
    let route = PathBuf::from(req.uri().path()).strip_prefix(route_prefix).unwrap().to_path_buf();

    // Check if route is in the route table
    if route_table.lock().await.contains_key(&route) {
        match req.method() {
            &hyper::Method::POST => {
                show_message( &mut CommandTerminal::new(), MessageType::Info, Message {
                    action: "POST",
                    details: route.to_str().unwrap(),
                });

                let bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
                let body = String::from_utf8(bytes.to_vec()).unwrap();

                let guard = route_table.lock().await;
                let topic_name = &guard.get(&route).unwrap().table_name;

                let res = configured_producer.lock().await.producer.send(
                    FutureRecord::to(topic_name)
                        .key(topic_name) // This should probably be generated by the client that pushes data to the API
                        .payload(&body)
                    , Timeout::After(Duration::from_secs(1))).await;

                match res {
                    Ok(_) => {
                        show_message( &mut CommandTerminal::new(), MessageType::Success, Message {
                            action: "SUCCESS",
                            details: route.to_str().unwrap(),
                        });
                        return Ok(Response::new("SUCCESS".to_string()))
                    },
                    Err(e) => {
                        println!("Error: {:?}", e);
                        return Ok(Response::new("ERROR".to_string()))
                    }
                }
            },
            _ => {
                show_message( &mut CommandTerminal::new(), MessageType::Info, Message {
                    action: "UNKNOWN METHOD",
                    details: route.to_str().unwrap(),
                });
                // If not, return a 404
                return Ok(Response::builder()
                    .status(StatusCode::METHOD_NOT_ALLOWED)
                    .body("Please use a POST method to send data to your ingestion point".to_string())?)
            }
        }
    }
    
    // If not, return a 404
    Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("NOTFOUND".to_string())?)
}

// TODO Figure out how to stop the web server
pub async fn start_webserver(term: &mut CommandTerminal, route_table: Arc<Mutex<HashMap::<PathBuf, RouteMeta>>>, redpanda_config: RedpandaConfig) {

    let addr = ([127, 0, 0, 1], 4000).into();

    show_message( term, MessageType::Info, Message {
        action: "starting",
        details: " server on port 4000",
    });

    let producer = Arc::new(Mutex::new(redpanda::create_producer(redpanda_config)));

    let main_service = make_service_fn(move |_| {
        let route_table = route_table.clone();
        let producer = producer.clone();

        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                handler(req, route_table.clone(), producer.clone())
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