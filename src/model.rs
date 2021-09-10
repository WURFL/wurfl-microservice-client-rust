/// Holds info about WURFL microservice running server
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
