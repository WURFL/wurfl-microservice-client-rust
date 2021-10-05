use std::borrow::BorrowMut;
use std::env;

use wmclient::*;
use std::collections::HashMap;

fn create_test_client() -> Result<WmClient, WmError> {
    let mut u_host = "localhost".to_string();
    let mut u_port = "8080".to_string();
    let mut host = u_host.as_str();
    let mut port = u_port.as_str();
    let env_host = env::var("WM_HOST");
    let env_port = env::var("WM_PORT");
    if env_host.is_ok() {
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
    let mut u_host = "localhost".to_string();
    let mut u_port = "8080".to_string();
    let mut host = u_host.as_str();
    let mut port = u_port.as_str();
    let env_host = env::var("WM_HOST");
    let env_port = env::var("WM_PORT");
    if env_host.is_ok() {
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
fn create_with_server_down_test() {
    let res = WmClient::new("http", "localhost", "18080", "");
    assert!(res.is_err());
}

#[test]
fn create_with_empty_server_values_test() {
    let res = WmClient::new("", "", "", "");
    assert!(res.is_err());
}

#[test]
fn test_get_info() {
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
fn test_has_static_capability() {
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let client = cl_res.unwrap();
    assert!(client.has_static_capability("brand_name"));
    assert!(client.has_static_capability("is_tablet"));
    assert!(!client.has_static_capability("unknown_static_cap"));
}

#[test]
fn test_has_virtual_capability() {
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let client = cl_res.unwrap();
    assert!(client.has_virtual_capability("form_factor"));
    assert!(client.has_virtual_capability("complete_device_name"));
    assert!(!client.has_virtual_capability("unknown_vcap"));
}

#[test]
fn test_lookup_useragent_ok() {
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let mut client = cl_res.unwrap();
    let ua = "Mozilla/5.0 (Linux; Android 7.0; SAMSUNG SM-G950F Build/NRD90M) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/5.2 Chrome/51.0.2704.106 Mobile Safari/537.36";
    let device_res = client.lookup_useragent(ua.to_string());
    assert!(device_res.is_ok());
    let device = device_res.unwrap();
    assert!(device.capabilities.len() > 0);
    assert_eq!(device.error, "");
    assert!(device.ltime.len() > 0);
    assert!(device.mtime > 0);
    assert_eq!("SM-G950F", device.capabilities.get("model_name").unwrap().as_str());
    assert_eq!("false", device.capabilities.get("is_robot").unwrap().as_str());
    assert_eq!("false", device.capabilities.get("is_full_desktop").unwrap().as_str());
}

#[test]
fn test_multiple_lookup_useragent(){
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let mut client = cl_res.unwrap();
    let ua = "Mozilla/5.0 (Linux; Android 7.0; SAMSUNG SM-G950F Build/NRD90M) AppleWebKit/537.36 (KHTML, like Gecko) SamsungBrowser/5.2 Chrome/51.0.2704.106 Mobile Safari/537.36";
    client.set_cache_size(100);
    for _i in 0..50 {
        let device_res = client.lookup_useragent(ua.to_string());
        assert!(device_res.is_ok());
        let device = device_res.unwrap();
        assert!(device.capabilities.len() > 0);
    }
    let sizes = client.get_actual_cache_sizes();
    // Cache is hit the first time, then it is always reused
    assert_eq!(1, sizes.1);

}

#[test]
fn test_lookup_empty_useragent() {
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let mut client = cl_res.unwrap();
    let device_res = client.lookup_useragent("".to_string());
    assert!(device_res.is_ok());
    let device = device_res.unwrap();
    assert!(device.capabilities.len() > 0);
    assert_eq!(device.error, "");
    assert!(device.ltime.len() > 0);
    assert!(device.mtime > 0);
    assert_eq!("generic", device.capabilities.get("wurfl_id").unwrap().as_str());
}


#[test]
fn test_lookup_useragent_with_specific_caps() {
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let mut client = cl_res.unwrap();
    let req_caps = vec! {"brand_name", "marketing_name", "is_full_desktop", "model_name"};
    client.set_requested_capabilities(Some(req_caps));
    let ua = "Mozilla/5.0 (Nintendo Switch; WebApplet) AppleWebKit/601.6 (KHTML, like Gecko) NF/4.0.0.5.9 NintendoBrowser/5.1.0.13341".to_string();
    let device_res = client.lookup_useragent(ua);
    assert!(device_res.is_ok());
    let device = device_res.unwrap();
    assert_eq!(device.capabilities.len(), 5);
    assert_eq!("Nintendo", device.capabilities.get("brand_name").unwrap().as_str());
    assert_eq!("Switch", device.capabilities.get("model_name").unwrap().as_str());
    assert_eq!("false", device.capabilities.get("is_full_desktop").unwrap().as_str());
}

#[test]
fn test_set_requested_capabilities() {
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let mut client = cl_res.unwrap();
    // In both static and vcap lists we add 1 correct name, 1 non existent name and 1 name that belongs to a different set
    client.set_requested_static_capabilities(Some(vec! {"brand_name", "invalid_name1", "is_ios"}));
    client.set_requested_virtual_capabilities(Some( vec! { "is_ios", "invalid_name2", "brand_name" }));

    let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 10_2_1 like Mac OS X) AppleWebKit/602.4.6 (KHTML, like Gecko) Version/10.0 Mobile/14D27 Safari/602.1";
    let dev_res = client.lookup_useragent(ua.to_string());
    assert!(dev_res.is_ok());
    let device = dev_res.unwrap();
    // 1 cap, 1 vcap + wurfl_id
    assert_eq!(3, device.capabilities.len());
    let cap = device.capabilities.get("invalid_name1");
    assert!(cap.is_none()); // this cap has been discarded because it does not exist
    client.set_requested_static_capabilities(None);
    let device_res = client.lookup_useragent(ua.to_string());
    assert!(device_res.is_ok());
    let device2 = device_res.unwrap();
    assert_eq!(2, device2.capabilities.len());
    client.set_requested_virtual_capabilities(None);
    let device_res2 = client.lookup_useragent(ua.to_string());
    assert!(device_res2.is_ok());
    let device3 = device_res2.unwrap();
    // resetting all required caps arrays, ALL available caps are returned
    assert!(device3.capabilities.len() > 10);
}

#[test]
fn test_reset_cache_on_requested_caps_change() {
    // Checks that cache is cleared whenever a reset of the requested static and/or virtual capabilities occur
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let mut client = client_res.unwrap();
    client.set_cache_size(2000);
    let req_caps: Vec<&str> = vec!{"brand_name", "is_wireless_device", "is_app"};
    client.set_requested_static_capabilities(Some(req_caps.clone()));
    let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 10_2_1 like Mac OS X) AppleWebKit/602.4.6 (KHTML, like Gecko) Version/10.0 Mobile/14D27 Safari/602.1";
    let device_res = client.lookup_useragent(ua.to_string());
    assert!(device_res.is_ok());

    let mut sizes = client.get_actual_cache_sizes();
    assert_eq!(0, sizes.0);
    assert_eq!(1, sizes.1);

    client.set_requested_capabilities(Some(req_caps));
    sizes = client.get_actual_cache_sizes();
    assert_eq!(0, sizes.0);
    assert_eq!(0, sizes.1);

    let _ =client.lookup_useragent(ua.to_string());
    sizes = client.get_actual_cache_sizes();
    assert_eq!(1, sizes.1);
    let req_caps2 = vec!{"brand_name", "is_wireless_device"};
    let req_vcaps = vec!{"is_app", "is_ios"};
    client.set_requested_static_capabilities(Some(req_caps2));
    client.set_requested_virtual_capabilities(Some(req_vcaps));
    sizes = client.get_actual_cache_sizes();
    assert_eq!(0, sizes.0);
    assert_eq!(0, sizes.1);
}

#[test]
fn test_lookup_headers_ok() {
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let mut client = client_res.unwrap();
    // Let's create test headers
    let mut headers: HashMap<String,String> = HashMap::new();
    headers.insert("X-Requested-With".to_string(),"json_client".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Accept-Encoding".to_string(), "gzip, deflate".to_string());
    headers.insert("X-UCBrowser-Device-UA".to_string(), "Mozilla/5.0 (SAMSUNG; SAMSUNG-GT-S5253/S5253DDJI7; U; Bada/1.0; en-us) AppleWebKit/533.1 (KHTML, like Gecko) Dolfin/2.0 Mobile WQVGA SMM-MMS/1.2.0 OPN-B".to_string());
    headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Nintendo Switch; WebApplet) AppleWebKit/601.6 (KHTML, like Gecko) NF/4.0.0.5.9 NintendoBrowser/5.1.0.13341".to_string());

    let device_res = client.lookup_headers(headers);
    assert!(device_res.is_ok());
    let device = device_res.unwrap();
    assert!(device.capabilities.len() > 0);
    assert_eq!("Samsung", device.capabilities.get("brand_name").unwrap().as_str());
    assert_eq!("GT-S5253", device.capabilities.get("model_name").unwrap().as_str());
    assert_eq!("false", device.capabilities.get("is_robot").unwrap().as_str());
}

#[test]
fn test_lookup_headers_with_specific_caps() {
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let mut client = client_res.unwrap();
    let req_caps: Vec<&str> = vec!{"brand_name", "is_full_desktop", "is_robot", "model_name"};
    client.set_requested_capabilities(Some(req_caps));
    // Let's create test headers
    let mut headers: HashMap<String,String> = HashMap::new();
    headers.insert("X-Requested-With".to_string(),"json_client".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers.insert("Accept-Encoding".to_string(), "gzip, deflate".to_string());
    headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Nintendo Switch; WebApplet) AppleWebKit/601.6 (KHTML, like Gecko) NF/4.0.0.5.9 NintendoBrowser/5.1.0.13341".to_string());

    let device_res = client.lookup_headers(headers);
    assert!(device_res.is_ok());
    let device = device_res.unwrap();
    assert_eq!(device.capabilities.len(), 5);
    assert_eq!("Nintendo", device.capabilities.get("brand_name").unwrap().as_str());
    assert_eq!("Switch", device.capabilities.get("model_name").unwrap().as_str());
    assert_eq!("false", device.capabilities.get("is_robot").unwrap().as_str());
}

#[test]
fn test_lookup_headers_with_mixed_case() {
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let mut client = client_res.unwrap();
    // Let's create test headers
    let mut headers: HashMap<String,String> = HashMap::new();
    headers.insert("X-Requested-With".to_string(),"json_client".to_string());
    headers.insert("CoNtent-TypE".to_string(), "application/json".to_string());
    headers.insert("accepT-ENcoDing".to_string(), "gzip, deflate".to_string());
    headers.insert("X-UCBrowser-Device-UA".to_string(), "Mozilla/5.0 (SAMSUNG; SAMSUNG-GT-S5253/S5253DDJI7; U; Bada/1.0; en-us) AppleWebKit/533.1 (KHTML, like Gecko) Dolfin/2.0 Mobile WQVGA SMM-MMS/1.2.0 OPN-B".to_string());
    headers.insert("UseR-AgEnt".to_string(), "Mozilla/5.0 (Nintendo Switch; WebApplet) AppleWebKit/601.6 (KHTML, like Gecko) NF/4.0.0.5.9 NintendoBrowser/5.1.0.13341".to_string());

    let device_res = client.lookup_headers(headers);
    assert!(device_res.is_ok());
    let device = device_res.unwrap();
    assert!(device.capabilities.len() > 0);
    assert_eq!("Samsung", device.capabilities.get("brand_name").unwrap().as_str());
    assert_eq!("GT-S5253", device.capabilities.get("model_name").unwrap().as_str());
    assert_eq!("false", device.capabilities.get("is_robot").unwrap().as_str());
}

#[test]
fn test_lookup_headers_with_empty_header_map() {
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let mut client = client_res.unwrap();
    // Passing an empty map should result in the creation of an empty request object, thus in a "generic" device detection...
    let headers: HashMap<String,String> = HashMap::new();

    let device_res = client.lookup_headers(headers);
    assert!(device_res.is_ok());
    let device = device_res.unwrap();
    assert!(device.capabilities.len() > 0);
    assert_eq!("generic", device.capabilities.get("wurfl_id").unwrap().as_str());
}

#[test]
fn test_single_lookup_device_id() {
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let mut client = client_res.unwrap();
    _internal_test_lookup_device_id(client.borrow_mut());
}

#[test]
fn test_multiple_lookup_device_id(){
    let cl_res = create_test_client();
    assert!(cl_res.is_ok());
    let mut client = cl_res.unwrap();
    client.set_cache_size(100);
    for _i in 0..50 {
        _internal_test_lookup_device_id(client.borrow_mut());
    }
    let sizes = client.get_actual_cache_sizes();
    // Cache is hit the first time, then it is always reused
    assert_eq!(1, sizes.0);

}

#[test]
fn test_lookup_wrong_device_id(){
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let mut client = client_res.unwrap();
    let result = client.lookup_device_id("doesnotexist".to_string());
    // wurfl is does not exist, method returns error
    assert!(result.is_err());
}

#[test]
fn test_get_all_oses(){
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let client = client_res.unwrap();
    let device_os_list_res = client.get_all_oses();
    assert!(device_os_list_res.is_ok());
    let os_list = device_os_list_res.unwrap();
    assert!(os_list.len() > 0);
    /*
    for os in os_list {
        println!("{}", os);
    }*/
}

#[test]
fn test_get_all_versions_for_os(){
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let client = client_res.unwrap();
    let os_versions_res = client.get_all_versions_for_os("iOS");
    assert!(os_versions_res.is_ok());
    let os_versions = os_versions_res.unwrap();
    assert!(os_versions.len() > 0);
    /*
    for v in os_versions {
        println!("{}", v);
    }*/
}

#[test]
fn test_get_all_versions_for_wrong_os_name(){
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let client = client_res.unwrap();
    // Gotcha: Apple is not an OS name
    let os_versions_res = client.get_all_versions_for_os("Apple");
    assert!(os_versions_res.is_err());
    let err = os_versions_res.err().unwrap();
    assert!(err.msg.len() > 0);
    assert!(err.msg.contains("Apple"));
    assert!(err.msg.contains("does not exist"));
}

#[test]
fn test_get_all_device_makes(){
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let client = client_res.unwrap();
    let makes_res = client.get_all_device_makes();
    assert!(makes_res.is_ok());
    let makes = makes_res.unwrap();
    assert!(makes.len() > 2000);
}

#[test]
fn test_get_all_devices_for_make(){
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let client = client_res.unwrap();
    let devices_res = client.get_all_devices_for_make("Nokia".to_string());
    assert!(devices_res.is_ok());
    let devices = devices_res.unwrap();
    assert!(devices.len() > 700);
    let d_opt = devices.get(0);
    assert!(d_opt.is_some());
    let d = d_opt.unwrap();
    assert!(d.model_name.len() > 0);

    // let' try with Apple
    let devices_res2 = client.get_all_devices_for_make("Apple".to_string());
    assert!(devices_res2.is_ok());
    let devices2 = devices_res2.unwrap();
    assert!(devices2.len() > 70);
}

#[test]
fn test_get_all_devices_for_not_existing_make(){
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let client = client_res.unwrap();
    let devices_res = client.get_all_devices_for_make("NotExisting".to_string());
    assert!(devices_res.is_err());
}

#[test]
fn test_set_http_timeout_expiration(){
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let mut client = client_res.unwrap();
    client.set_http_timeout(1);
    let res = client.get_info();
    assert!(res.is_err());
    let err_opt = res.err();
    assert!(err_opt.is_some());
    let err_msg = err_opt.unwrap();
    assert!(err_msg.to_string().contains("timed out"));
}

#[test]
fn test_set_http_timeout(){
    let client_res = create_test_client();
    assert!(client_res.is_ok());
    let mut client = client_res.unwrap();
    client.set_http_timeout(5000);
    let res = client.get_info();
    assert!(res.is_ok());
}

// we reuse this for several tests
fn _internal_test_lookup_device_id(client: &mut WmClient){
    let device_res = client.lookup_device_id("nokia_generic_series40".to_string());
    assert!(device_res.is_ok());
    let device = device_res.unwrap();
    assert!(device.capabilities.len() > 0);
    assert_eq!("true", device.capabilities.get("is_mobile").unwrap().as_str());
    assert_eq!("Feature Phone", device.capabilities.get("form_factor").unwrap().as_str());
}

