use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use wmclient::WmClient;

#[tokio::main]
pub async fn main() {
    // First, create a Wurfl microservice client instance and set a cache for it.
    // change your server data to your preferred settings
    let wmclient_res = WmClient::new("http", "localhost", "8080","");
    let wm_client = match wmclient_res {
        Ok(wm_client) => wm_client,
        Err(error) => panic!("Problem initializing wurfl microservice client: {:?}", error),
    };
    println!("Created WURFL microservice client API for Rust version: {}", wm_client.get_api_version());
    // The wurfl microservice client is mutable because contains some updatable internal state, so we need to
    // wrap it into a Mutex to use it in the detect function
    let safe_wm_client = Arc::new(Mutex::new(wm_client));
    // A `Service` is needed for every connection, so this
    // creates one wrapping our `detect` function.
    let make_svc = make_service_fn(move |_conn| {
        let mut safe_wm_client_clone = Arc::clone(&safe_wm_client);
        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                let response = detect(req, &mut safe_wm_client_clone);
                async { Ok::<_, Infallible>(Response::new(Body::from(response))) }
            }))
        }
    });

    // We'll bind the server to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::bind(&addr).serve(make_svc);
    match server.await {
        Err(_) => panic!("An error occurred while running WURFL microservice hyper server example, shutting down"),
        _ => (),
    }
}

// Actual device detection: returns a string with wurfl_id and virtual capability complete_device_name
fn detect(_req: Request<Body>, safe_client: &mut Arc<Mutex<WmClient>>) -> String {
    let mut client_guard = safe_client.lock().unwrap();
    let device = match client_guard.lookup_headers(_req.headers()) {
        Ok(d) => d,
        Err(_) => panic!("Error during lookup")
    };
    drop(client_guard);
    let body = format!("Detected device: {} - {} ", device.capabilities.get("wurfl_id").unwrap(),
                       device.capabilities.get("complete_device_name").unwrap());
    return body;
}