const DEVICE_ID_CACHE_TYPE: &str = "dId-cache";
const USERAGENT_CACHE_TYPE: &str = "ua-cache";
const DEFAULT_CONTENT_TYPE: &str = "application/json";

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
    _dev_id_cache: Mutex<LruCache<String, JSONDeviceData>>,
    // Maps device ID -> JSONDeviceData
    _ua_cache: Mutex<LruCache<String, JSONDeviceData>>,
    // Maps concat headers (mainly UA) -> JSONDeviceData
    // Stores the result of time consuming call getAllMakeModel
    _make_models: Mutex<Vec<JSONMakeModel>>,
    // List of device manufacturers
    _device_makes: Mutex<Vec<String>>,
    _device_makes_map: HashMap<String, Vec<JSONModelMktName>>,
    // Map that associates os name to JSONDeviceOsVersions objects
    _device_os_versions_map: HashMap<String, Vec<String>>,
    // List of all device OSes
    _device_oses: Mutex<Vec<String>>,
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
        let mut d_oses = vec![];
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
            _dev_id_cache: Mutex::new(d_id_cache),
            _ua_cache: Mutex::new(ua_cache),
            _make_models: Mutex::new(mk_md),
            _device_makes: Mutex::new(d_mk),
            _device_makes_map: d_mm,
            _device_os_versions_map: d_ovm,
            _device_oses: Mutex::new(d_oses),
            _ltime: "0".to_string(),
        };

        let info_res = wm_client.get_info();
        if info_res.is_ok() {
            let info = info_res.unwrap();
            wm_client.important_headers = info.important_headers.clone();
            wm_client.static_caps = info.static_caps.clone();
            wm_client.static_caps.sort();
            wm_client.virtual_caps = info.virtual_caps.clone();
            wm_client.virtual_caps.sort();
            wm_client._ltime = info.ltime;
            return Ok(wm_client);
        } else {
            return Err(WmError { msg: "Unable to create WURFL Microservice client: unable to get info from WM server".to_string() });
        }
    }

    /// Returns the version of this Rust client API
    pub fn get_api_version(&self) -> &str {
        return "1.0.0";
    }

    pub fn has_static_capability(&self, cap_name: &str) -> bool {
        return self.static_caps.contains(&cap_name.to_string());
    }

    pub fn has_virtual_capability(&self, vcap_name: &str) -> bool {
        return self.virtual_caps.contains(&vcap_name.to_string());
    }

    /// Returns info about the running WURFL Microservice server to which this client is connected
    pub fn get_info(&self) -> Result<JSONInfoData, WmError> {
        let url = self.create_url("/v2/getinfo/json");
        match ureq::get(url.as_str()).set("content-type", DEFAULT_CONTENT_TYPE)
            .call() {
            Ok(res) => {
                let info_res = res.into_json();
                return if info_res.is_ok() {
                    Ok(info_res.unwrap())
                } else {
                    Err(WmError { msg: "Unable to create Wurfl microservice client: could not parse server info".to_string() })
                };
            }
            Err(ierr) => {
                return Err(WmError { msg: format!(" Unable to create Wurfl microservice client: {}", ierr.to_string()) });
            }
        };
    }

    // LookupUserAgent - Searches WURFL device data using the given user-agent for detection
    pub fn lookup_useragent(&self, user_agent: String) -> Result<JSONDeviceData, WmError> {

        // First: cache lookup
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), user_agent);

        let cache_lock = self._ua_cache.lock();
        if cache_lock.is_ok() {
            let mut guard = cache_lock.unwrap();
            let device_opt = guard.get(&self.get_user_agent_cache_key(headers.clone()).unwrap());
            if device_opt.is_some() {
                let device_ref = device_opt.unwrap();
                let device = JSONDeviceData{
                    capabilities: device_ref.capabilities.clone(),
                    error: device_ref.error.clone(),
                    mtime: device_ref.mtime.clone(),
                    ltime: device_ref.ltime.clone()
                };
                return Ok(device);
            }
            // drop the guard will unlock cache
            drop(guard);
        }

        let json_request = Request::new(headers.clone(),
                                        self.requested_static_caps.clone(),
                                        self.requested_virtual_caps.clone(), None);
        let result = self._internalLookup(json_request, "/v2/lookupuseragent/json".to_string());
        if result.is_ok() {
            let device = result.unwrap();

            // check if server WURFL.xml has been updated and, if so, clear caches
            //c.clearCachesIfNeeded(deviceData.Ltime) // TODO

            // we need to lock when writing since cache is not thread safe
            let response_cache_lock = self._ua_cache.lock();
            if response_cache_lock.is_ok() {
                let mut guard = response_cache_lock.unwrap();
                guard.put(self.get_user_agent_cache_key(headers.clone()).unwrap(), device.clone());
                drop(guard);
            }
            return Ok(device);
        } else {
            return Err(result.err().unwrap());
        }
    }

    fn clear_caches(&mut self) {
        let ua_lock_res = self._ua_cache.lock();
        if ua_lock_res.is_ok() {
            let mut ua_cache = ua_lock_res.unwrap();
            if ua_cache.len() > 0 {
                ua_cache.clear();
            }
        }

        let dev_lock_res = self._dev_id_cache.lock();
        if dev_lock_res.is_ok() {
            let mut dev_id_cache = dev_lock_res.unwrap();
            if dev_id_cache.len() > 0 {
                dev_id_cache.clear();
            }
        }

        let mk_md_lock_res = self._make_models.lock();
        if mk_md_lock_res.is_ok() {
            let mut make_models = mk_md_lock_res.unwrap();
            make_models.clear();
        }

        let dev_makes_lock_res = self._device_makes.lock();
        if dev_makes_lock_res.is_ok() {
            let mut device_makes = dev_makes_lock_res.unwrap();
            device_makes.clear();
            self._device_makes_map.clear();
        }

        let dev_os_lock_res = self._device_oses.lock();
        if dev_os_lock_res.is_ok() {
            let mut device_oses = dev_os_lock_res.unwrap();
            device_oses.clear();
        }
    }

    /// Sets the new cache size. Changing cache size will result in a cache purge.
    pub fn set_cache_size(&mut self, ua_max_entries: usize) {
        self._ua_cache = Mutex::new(LruCache::new(ua_max_entries));
    }

    fn create_url(&self, path: &str) -> String {
        if !self._base_uri.is_empty() {
            return format!("{}://{}:{}/{}{}", self._scheme.as_str(), self._host.as_str(), self._port.as_str(), self._base_uri.as_str(), path);
        }
        return format!("{}://{}:{}{}", self._scheme.as_str(), self._host.as_str(), self._port.as_str(), path);
    }

    fn get_user_agent_cache_key(&self, headers: HashMap<String, String>) -> Option<String> {
        let mut key = String::new();
        // Using important headers array preserves header name order
        for hname in &self.important_headers {
            if !hname.is_empty() {
                let h_val = headers.get(hname.as_str());
                if h_val.is_some() {
                    key = key + h_val.unwrap().as_str();
                }
            }
        }
        let digest = md5::compute(key);
        let str_digest = format!("{:x}", digest);
        return Some(str_digest);
    }

    fn get_wm_client_user_agent(&self) -> String {
        let mut ua = String::new();
        ua = ua + "rust-wmclient-api-" + self.get_api_version();
        return ua;
    }

    // Performs a GET request and returns the response body as a JSON String that can be unmarshalled
    fn _internal_get(&self, endpoint: String) -> Result<String, WmError> {
        let url = self.create_url(endpoint.as_str());
        match ureq::get(url.as_str()).set("content-type", DEFAULT_CONTENT_TYPE)
            .call() {
            Ok(res) => {
                let result = res.into_string();
                return if result.is_ok() {
                    Ok(result.unwrap())
                } else {
                    let err = result.err().unwrap();
                    let msg = format!("Unable to perform get for path {}. Error {}", url, err.to_string());
                    Err(WmError { msg })
                };
            }
            Err(e) => {
                let msg = format!("Unable to perform get for path {}. Error {}", url, e.to_string());
                return Err(WmError { msg });
            }
        }
    }

    fn _internalLookup(&self, request: Request, path: String) -> Result<JSONDeviceData, WmError> {
        let url = self.create_url(path.as_str());
        let json_req = ureq::json!(request);
        let resp_res = ureq::post(url.as_str())
            .set("Content-type", DEFAULT_CONTENT_TYPE)
            .set("User-Agent", self.get_wm_client_user_agent().as_str())
            .send_json(json_req);

        if resp_res.is_ok() {
            let resp = resp_res.unwrap();
            let str_resp_res = resp.into_string();
            if str_resp_res.is_ok() {
                let str_resp = str_resp_res.unwrap();
                let device_res: Result<JSONDeviceData, serde_json::Error> = serde_json::from_str(str_resp.as_str());
                if device_res.is_ok() {
                    let device = device_res.unwrap();
                    let result = Ok(device);
                    return result;
                } else {
                    let serde_err = device_res.err();
                    if serde_err.is_some() {
                        return Err(WmError{ msg: serde_err.unwrap().to_string() });
                    } else {
                        return Err(WmError{msg: "Unable to parse JSON response".to_string() })
                    }
                }
            } else {
                return Err(WmError { msg: str_resp_res.err().unwrap().to_string() });
            }
        } else {
            return Err(WmError { msg: resp_res.err().unwrap().to_string() });
        }
    }
}
