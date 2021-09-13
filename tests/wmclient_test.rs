use wmclient::*;
use std::env;

#[test]
fn create_ok_test() {
    let mut u_host= "localhost".to_string();
    let mut u_port= "8080".to_string();
    let mut host = u_host.as_str();
    let mut port = u_port.as_str();
    let env_host = env::var("WM_HOST");
    let env_port = env::var("WM_PORT");
    if env_host.is_ok(){
        u_host = env_host.unwrap().to_owned();
        host = u_host.as_str();
    }
    if env_port.is_ok() {
        u_port = env_port.unwrap().to_owned();
        port = u_port.as_str();
    }
    let client_res = WmClient::new("http", host, port, "");
    assert!(client_res.is_ok());
    let client = client_res.unwrap();
    assert!(client.important_headers.len() > 0);
    assert!(client.static_caps.len() > 0);
    assert!(client.virtual_caps.len() > 0);
}