use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use wmclient::WmClient;

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
pub async fn main() {
    let safe_wm_client = tokio::task::spawn_blocking(move || {
        create_wm_client()
    }).await
    .expect("WM client creation failed");

    let make_svc = make_service_fn(move |_conn| {
        let safe_wm_client_clone = Arc::clone(&safe_wm_client);
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