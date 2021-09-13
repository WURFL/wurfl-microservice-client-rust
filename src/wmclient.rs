const DEVICE_ID_CACHE_TYPE: &str = "dId-cache";
const USERAGENT_CACHE_TYPE: &str = "ua-cache";

pub struct WmClient {
    _scheme: String,
    _host: String,
    _port: String,
    _base_uri: String,

    // These are the lists of all static or virtual that can be returned by the running wm server
    pub static_caps: Vec<String>,
    pub virtual_caps: Vec<String>,

    // Requested are used in the lookup requests, accessible via the SetRequested[...] functions
    requested_static_caps: Vec<String>,
    requested_virtual_caps: Vec<String>,
    pub important_headers: Vec<String>,
    // Internal caches
    _dev_id_cache: LruCache<String, JSONDeviceData>,
    // Maps device ID -> JSONDeviceData
    _ua_cache: LruCache<String, JSONDeviceData>,
    // Maps concat headers (mainly UA) -> JSONDeviceData
    // Stores the result of time consuming call getAllMakeModel
    _make_models: Vec<JSONMakeModel>,
    // List of device manufacturers
    _device_makes: Vec<String>,
    _device_makes_map: HashMap<String, Vec<JSONModelMktName>>,
    // Map that associates os name to JSONDeviceOsVersions objects
    _device_os_versions_map: HashMap<String, Vec<String>>,
    // List of all device OSes
    _device_oses: Vec<String>,
    _ltime: String,
}

impl WmClient {
    /// Creates a new instance of the WURFL microservice client
    pub fn new(scheme: &str, host: &str, port: &str, base_uri: &str) -> Result<WmClient, WmError> {
        let mut d_id_cache = LruCache::new(20000);
        let mut ua_cache = lru::LruCache::new(200000);
        let mut st_cap = vec![];
        let mut req_st_cap = vec![];
        let mut req_v_cap = vec![];
        let mut v_cap = vec![];
        let mut i_h = vec![];
        let mut mk_md = vec![];
        let mut d_mk = vec![];
        let mut d_mm = HashMap::new();
        let mut d_ovm = HashMap::new();
        let mut d_oses= vec![];
        let mut wm_client = WmClient {
            _scheme: scheme.to_string(),
            _host: host.to_string(),
            _port: port.to_string(),
            _base_uri: base_uri.to_string(),
            static_caps: st_cap,
            virtual_caps: v_cap,
            requested_static_caps: req_st_cap,
            requested_virtual_caps: req_v_cap,
            important_headers: i_h,
            _dev_id_cache: d_id_cache,
            _ua_cache: ua_cache,
            _make_models: mk_md,
            _device_makes: d_mk,
            _device_makes_map: d_mm,
            _device_os_versions_map: d_ovm,
            _device_oses: d_oses,
            _ltime: "0".to_string(),
        };

        let info_res = wm_client.get_info();
        if info_res.is_ok(){
            let info = info_res.unwrap();
            wm_client.important_headers = info.important_headers.clone();
            wm_client.static_caps = info.static_caps.clone();
            wm_client.static_caps.sort();
            wm_client.virtual_caps = info.virtual_caps.clone();
            wm_client.virtual_caps.sort();
            wm_client._ltime = info.ltime;
            return Ok(wm_client)
        }
        else {
            return Err(WmError{msg: "Unable to create WURFL Microservice client: unable to get info from WM server".to_string() })
        }
    }

    pub fn get_info(&self) -> Result<JSONInfoData,WmError> {
        let url = self.create_url("/v2/getinfo/json");
        let info_res = match ureq::get(url.as_str()).set("content-type", "application/json")
            .call() {
            Ok(res) => {
                let info_res = res.into_json();
                return if info_res.is_ok() {
                    Ok(info_res.unwrap())
                } else {
                    Err(WmError { msg: "Unable to create Wurfl microservice client: could not parse server info".to_string() })
                }
            }
            Err(ierr) => {
                return Err(WmError{ msg: format!(" Unable to create Wurfl microservice client: {}", ierr.to_string())});
            }
        };
    }

    fn create_url(&self, path: &str) -> String {
        if !self._base_uri.is_empty() {
            return format!("{}://{}:{}/{}{}", self._scheme.as_str(),self._host.as_str(),self._port.as_str(),self._base_uri.as_str(), path);
        }
        return format!("{}://{}:{}{}", self._scheme.as_str(),self._host.as_str(),self._port.as_str(), path);
    }
}
