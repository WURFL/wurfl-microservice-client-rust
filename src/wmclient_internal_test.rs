// WHY THESE TESTS ARE HERE AND NOT IN THE tests/ DIRECTORY
// Files in tests/ are compiled as separate crates and can only use the public API of WmClient.
// The tests below must simulate a server that has reloaded its WURFL file, i.e. the ltime known by the
// client is different from the one the server sends with the next lookup response. A real server cannot
// be told to do it on demand, so the tests set the private field `_ltime` of WmClient, which is only
// possible from inside the crate. This module is compiled only for `cargo test` (see lib.rs).
// Like the tests in tests/, they use a live WM server (WM_HOST / WM_PORT, default localhost:8080)
// and no mocks.
use super::*;

const UA_1: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1";
const UA_2: &str = "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Mobile Safari/537.36";
const STALE_LTIME: &str = "stale ltime: the WURFL file was loaded again by the server";

fn test_client() -> WmClient {
    let host = std::env::var("WM_HOST").unwrap_or("localhost".to_string());
    let port = std::env::var("WM_PORT").unwrap_or("8080".to_string());
    let mut client = WmClient::new("http", host.as_str(), port.as_str(), "").unwrap();
    client.set_cache_size(100);
    client
}

fn ua_headers(ua: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), ua.to_string());
    headers
}

// The server reports a load time (ltime) that differs from the one the client knows: the client must
// understand that WURFL has been reloaded, so it clears its caches and stores the new ltime.
#[test]
fn lookup_headers_clears_caches_when_server_ltime_changes() {
    let mut client = test_client();
    let server_ltime = client._ltime.clone();

    client.lookup_headers(ua_headers(UA_1)).unwrap();
    assert_eq!((0, 1), client.get_actual_cache_sizes());

    // as if the server had reloaded the WURFL file after the client got its ltime
    client._ltime = STALE_LTIME.to_string();
    client.lookup_headers(ua_headers(UA_2)).unwrap();

    assert_eq!(server_ltime, client._ltime);
    // the entry cached before the reload is gone, only the new one is there
    assert_eq!((0, 1), client.get_actual_cache_sizes());
}

#[test]
fn lookup_useragent_clears_caches_when_server_ltime_changes() {
    let mut client = test_client();
    let server_ltime = client._ltime.clone();

    client.lookup_useragent(UA_1.to_string()).unwrap();
    client._ltime = STALE_LTIME.to_string();
    client.lookup_useragent(UA_2.to_string()).unwrap();

    assert_eq!(server_ltime, client._ltime);
    assert_eq!((0, 1), client.get_actual_cache_sizes());
}

#[test]
fn lookup_device_id_clears_caches_when_server_ltime_changes() {
    let mut client = test_client();
    let server_ltime = client._ltime.clone();

    client.lookup_device_id("apple_iphone_ver17".to_string()).unwrap();
    assert_eq!((1, 0), client.get_actual_cache_sizes());
    client._ltime = STALE_LTIME.to_string();
    client.lookup_device_id("generic".to_string()).unwrap();

    assert_eq!(server_ltime, client._ltime);
    assert_eq!((1, 0), client.get_actual_cache_sizes());
}
