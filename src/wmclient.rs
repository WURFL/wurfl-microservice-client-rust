const DEVICE_ID_CACHE_TYPE: &str = "dId-cache";
const USERAGENT_CACHE_TYPE: &str = "ua-cache";

struct WmClient {
    scheme: String,
    host: String,
    port: String,
    uri: String,

    // These are the lists of all static or virtual that can be returned by the running wm server
    static_caps: Vec<String>,
    virtual_caps: Vec<String>,

    // Requested are used in the lookup requests, accessible via the SetRequested[...] functions
    requested_static_caps: Vec<String>,
    requested_virtual_caps: Vec<String>,
    important_headers: Vec<String>,

    // Internal caches
    _dev_id_cache: LruCache<String, JSONDeviceData>,
    // Maps device ID -> JSONDeviceData
    _ua_cache: LruCache<String, JSONDeviceData>, // Maps concat headers (mainly UA) -> JSONDeviceData
}

impl WmClient {
    /// Creates a new instance of
    pub fn new() {}
}
