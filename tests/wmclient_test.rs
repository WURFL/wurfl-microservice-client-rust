use wmclient::*;
use std::env;

fn create_test_client() -> Result<WmClient, WmError> {

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
    return WmClient::new("http", host, port, "");
}

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

#[test]
fn  create_with_server_down_test() {
    let res = WmClient::new("http", "localhost", "18080", "");
    assert!(res.is_err());
}

#[test]
fn create_with_empty_server_values_test() {
    let res = WmClient::new("", "", "", "");
    assert!(res.is_err());
}

#[test]
fn get_info_test() {
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let client = cl_res.unwrap();
    let info_res = client.get_info();
    assert!(info_res.is_ok());
    let info = info_res.unwrap();
    assert!(info.wurfl_api_version.len() > 0);
    assert!(info.important_headers.len() > 0);
    assert!(info.static_caps.len() > 0);
    assert!(info.virtual_caps.len() > 0);
}

#[test]
fn has_static_capability_test(){
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let client = cl_res.unwrap();
    assert!(client.has_static_capability("brand_name"));
    assert!(client.has_static_capability("is_tablet"));
    assert!(!client.has_static_capability("unknown_static_cap"));
}

#[test]
fn has_virtual_capability_test(){
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let client = cl_res.unwrap();
    assert!(client.has_virtual_capability("form_factor"));
    assert!(client.has_virtual_capability("complete_device_name"));
    assert!(!client.has_virtual_capability("unknown_vcap"));
}