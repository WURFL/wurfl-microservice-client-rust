use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Holds info about WURFL microservice running server
#[derive(Debug, Serialize, Deserialize)]
pub struct JSONInfoData {
    pub wurfl_api_version: String,
    pub wm_version: String,
    pub wurfl_info: String,
    pub important_headers: Vec<String>,
    pub static_caps: Vec<String>,
    pub virtual_caps: Vec<String>,
    ltime: String,
}

/// Holds the detected device data received from WURFL Microservice server.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JSONDeviceData {
    pub capabilities: HashMap<String, String>,
    pub error: String,
    pub mtime: i64,
    pub ltime: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JSONMakeModel {
    brand_name: String,
    model_name: String,
    marketing_name: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct  JSONModelMktName {
    pub model_name: String,
    pub marketing_name: String
}

#[derive(Debug, Serialize, Deserialize)]
struct  JSONDeviceOsVersions {
    device_os: String,
    device_os_version: String,
}

/// Request - data object that is sent to the WM server in POST requests
#[derive(Debug, Serialize)]
struct Request {
    lookup_headers: Option<HashMap<String, String>>,
    requested_caps: Option<Vec<String>>,
    requested_vcaps: Option<Vec<String>>,
    wurfl_id: Option<String>,
}

impl Request {
    pub fn new(lh: Option<HashMap<String, String>>, req_caps: Option<Vec<String>>, req_vcaps: Option<Vec<String>>, wid: Option<String>) -> Request {
        return Request {
            lookup_headers: lh,
            requested_caps: req_caps,
            requested_vcaps: req_vcaps,
            wurfl_id: wid
        };
    }
}

// Custom error for WURFL handle operations
#[derive(Error, Debug)]
pub struct WmError {
    pub msg: String,
}

impl std::fmt::Display for WmError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}