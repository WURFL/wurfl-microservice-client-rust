use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use wmclient::WmClient;

/// Creates a new `WmClient` instance and returns it wrapped in an `Arc<Mutex<WmClient>>`.
///
/// This function initializes a new WURFL microservice client API for Rust. It takes no arguments and
/// returns the client instance wrapped in a thread-safe `Arc<Mutex<WmClient>>`. If there is an error
/// initializing the client, the function will panic with the error message.
///
/// The created client instance is printed to the console, displaying the API version.
fn create_wm_client() -> Arc<Mutex<WmClient>> {
    let wmclient_res = WmClient::new("http", "localhost", "8080","");
    let wm_client = match wmclient_res {
        Ok(wm_client) => wm_client,
        Err(error) => panic!("Problem initializing wurfl microservice client: {:?}", error),
    };
    println!("Created WURFL microservice client API for Rust version: {}", wm_client.get_api_version());
    Arc::new(Mutex::new(wm_client))
}

#[tokio::main]
/// Runs a web server that detects device capabilities using the WURFL microservice client.
///
/// This function creates a new WURFL microservice client, starts a Hyper web server on `localhost:3000`,
/// and handles incoming requests by detecting the device capabilities using the WURFL client.
/// The detected device information is returned in the response body.
/// Since both creation and APIs of the wmclient and the web server are blocking, we need to use
/// the spawn_blocking function to move its usage to a separate thread.
pub async fn main() {
    let safe_wm_client = tokio::task::spawn_blocking(move || {
        create_wm_client()
    }).await
    .expect("WM client creation failed");

    let make_svc = make_service_fn(move |_conn| {
        let safe_wm_client_clone = Arc::clone(&safe_wm_client);
        // This is an async block that will be executed as part of the service function.
        async {
            Ok::<_, Infallible>(service_fn(move |req| {
                let safe_wm_client_clone = Arc::clone(&safe_wm_client_clone);
                async move {
                    let response = detect(req, safe_wm_client_clone).await;
                    Ok::<_, Infallible>(Response::new(Body::from(response)))
                }
            }))
        }
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server = Server::bind(&addr).serve(make_svc);
    if let Err(_) = server.await {
        eprintln!("An error occurred while running WURFL microservice hyper server example, shutting down");
    }
}
/// Detects the device information from the request headers and returns a formatted string with the device capabilities.
///
/// This function takes a request object and a thread-safe WURFL microservice client instance, and uses the client to look up the device information based on the request headers. It then formats the detected device information as a string and returns it.
///
/// # Arguments
/// * `req` - The HTTP request object containing the headers to detect the device.
/// * `safe_client` - A thread-safe WURFL microservice client instance to use for the device lookup.
///
/// # Returns
/// A string containing the detected device information, including the WURFL ID and the complete device name.
async fn detect(req: Request<Body>, safe_client: Arc<Mutex<WmClient>>) -> String {
    tokio::task::spawn_blocking(move || {
        let mut client_guard = safe_client.lock().unwrap();
        let device = match client_guard.lookup_headers(req.headers()) {
            Ok(d) => d,
            Err(_) => panic!("Error during lookup")
        };
        format!("Detected device: {} - {} ", 
                device.capabilities.get("wurfl_id").unwrap(),
                device.capabilities.get("complete_device_name").unwrap())
    }).await.unwrap()
}