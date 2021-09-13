use serde::{Serialize, Deserialize};
use thiserror::Error;

/// Holds info about WURFL microservice running server
#[derive(Deserialize)]
struct JSONInfoData {
    wurfl_api_version: String,
    wm_version: String,
    wurfl_info: String,
    important_headers: Vec<String>,
    static_caps: Vec<String>,
    virtual_caps: Vec<String>,
    ltime: String,
}

/// Holds the detected device data received from WURFL Microservice server.
struct JSONDeviceData {
    capabilities: HashMap<String, String>,
    error: String,
    m_time: i64,
    l_time: String,
}

struct JSONMakeModel {
    brand_name: String,
    model_name: String,
    marketing_name: String
}

struct  JSONModelMktName {
    model_name: String,
    marketing_name: String
}

struct  JSONDeviceOsVersions {
    device_os: String,
    device_os_version: String,
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
