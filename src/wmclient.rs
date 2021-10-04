pub const DEVICE_ID_CACHE_TYPE: &str = "dId-cache";
pub const USERAGENT_CACHE_TYPE: &str = "ua-cache";
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
    requested_static_caps: Option<Vec<String>>,
    requested_virtual_caps: Option<Vec<String>>,
    pub important_headers: Vec<String>,
    // Internal caches
    _cache: Cache,
    // Maps concat headers (mainly UA) -> JSONDeviceData
    // Stores the result of time consuming call getAllMakeModel
    _make_models: Mutex<Vec<JSONMakeModel>>,
    // List of device manufacturers
    _device_makes: Mutex<Vec<String>>,
    _device_makes_map: Mutex<HashMap<String, Vec<JSONModelMktName>>>,
    // Map that associates os name to JSONDeviceOsVersions objects
    _device_os_versions_map: Mutex<HashMap<String, Vec<String>>>,
    // List of all device OSes
    _device_oses: Mutex<Vec<String>>,
    _ltime: String,
}

impl WmClient {
    /// Creates a new instance of the WURFL microservice client
    pub fn new(scheme: &str, host: &str, port: &str, base_uri: &str) -> Result<WmClient, WmError> {
        let st_cap = vec![];
        let req_st_cap = vec![];
        let req_v_cap = vec![];
        let v_cap = vec![];
        let i_h = vec![];
        let mk_md = vec![];
        let d_mk = vec![];
        let d_mm = HashMap::new();
        let d_ovm = HashMap::new();
        let d_oses = vec![];
        let mut wm_client = WmClient {
            _scheme: scheme.to_string(),
            _host: host.to_string(),
            _port: port.to_string(),
            _base_uri: base_uri.to_string(),
            static_caps: st_cap,
            virtual_caps: v_cap,
            requested_static_caps: Some(req_st_cap),
            requested_virtual_caps: Some(req_v_cap),
            important_headers: i_h,
            _cache: Cache::new(50000),
            _make_models: Mutex::new(mk_md),
            _device_makes: Mutex::new(d_mk),
            _device_makes_map: Mutex::new(d_mm),
            _device_os_versions_map: Mutex::new(d_ovm),
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
    pub fn lookup_useragent(&mut self, user_agent: String) -> Result<JSONDeviceData, WmError> {

        // First: cache lookup
        let mut headers = HashMap::new();
        headers.insert("User-Agent".to_string(), user_agent);
        let device_opt = self._cache.get(USERAGENT_CACHE_TYPE.to_string(), self.get_user_agent_cache_key(headers.clone()).unwrap());
        if device_opt.is_some() {
            let device_ref = device_opt.unwrap();
            let device = JSONDeviceData {
                capabilities: device_ref.capabilities.clone(),
                error: device_ref.error.clone(),
                mtime: device_ref.mtime.clone(),
                ltime: device_ref.ltime.clone(),
            };
            return Ok(device);
        }

        let json_request = Request::new(Some(headers.clone()),
                                        self.requested_static_caps.clone(),
                                        self.requested_virtual_caps.clone(), None);
        let result = self._internal_lookup(json_request, "/v2/lookupuseragent/json".to_string());
        if result.is_ok() {
            let device = result.unwrap();

            // check if server WURFL.xml has been updated and, if so, clear caches
            self._clear_caches_if_needed(device.ltime.clone());
            self._cache.put(USERAGENT_CACHE_TYPE.to_string(), self.get_user_agent_cache_key(headers.clone()).unwrap(), device.clone());
            return Ok(device);
        } else {
            return Err(result.err().unwrap());
        }
    }

    /// lookup_device_id - Searches WURFL device data using its wurfl_id value
    pub fn lookup_device_id(&mut self, device_id: String) -> Result<JSONDeviceData, WmError> {

        // First: cache lookup
        let device_opt = self._cache.get(DEVICE_ID_CACHE_TYPE.to_string(), device_id.clone());
        if device_opt.is_some() {
            let device_ref = device_opt.unwrap();
            let device = JSONDeviceData {
                capabilities: device_ref.capabilities.clone(),
                error: device_ref.error.clone(),
                mtime: device_ref.mtime.clone(),
                ltime: device_ref.ltime.clone(),
            };
            return Ok(device);
        }

        let json_request = Request::new(None,
                                        self.requested_static_caps.clone(),
                                        self.requested_virtual_caps.clone(), Some(device_id.clone()));
        let result = self._internal_lookup(json_request, "/v2/lookupdeviceid/json".to_string());
        if result.is_ok() {
            let device = result.unwrap();

            // check if server WURFL.xml has been updated and, if so, clear caches
            self._clear_caches_if_needed(device.ltime.clone());

            self._cache.put(DEVICE_ID_CACHE_TYPE.to_string(), device_id.clone(), device.clone());
            return Ok(device);
        } else {
            return Err(result.err().unwrap());
        }
    }

    /// LookupHeaders - detects a device and returns its data in JSON format
    pub fn lookup_headers(&self, in_headers: HashMap<String, String>) -> Result<JSONDeviceData, WmError> {
        let mut headers: HashMap<String, String> = HashMap::new();

        // first: make all headers lowercase
        let mut lower_key_map: HashMap<String, String> = HashMap::new();
        for item in in_headers {
            lower_key_map.insert(item.0.to_lowercase(), item.1);
        }

        // copy important headers with the headers name properly cased.
        let ihs = self.important_headers.clone();
        for ih_name in ihs {
            let h_value = lower_key_map.get(ih_name.to_lowercase().as_str());
            if h_value.is_some() && !h_value.unwrap().is_empty() {
                headers.insert(ih_name, h_value.unwrap().to_string());
            }
        }
        // Create the request object
        let mut request = Request::new(Some(headers.clone()), self.requested_static_caps.clone(), self.requested_virtual_caps.clone(), None);

        // Do a cache lookup
        let device_opt = self._cache.get(USERAGENT_CACHE_TYPE.to_string(), self.get_user_agent_cache_key(headers.clone()).unwrap());

        if device_opt.is_some() {
            let d = device_opt.unwrap();
            let device = JSONDeviceData {
                capabilities: d.capabilities.clone(),
                error: d.error.clone(),
                mtime: d.mtime.clone(),
                ltime: d.ltime.clone(),
            };
            return Ok(device);
        }

        request.requested_caps = self.requested_static_caps.clone();
        request.requested_vcaps = self.requested_virtual_caps.clone();


        let device_res = self._internal_lookup(request, "/v2/lookuprequest/json".to_string());
        if device_res.is_ok() {
            let device = device_res.unwrap();
            // check if server WURFL.xml has been updated and, if so, clear caches
            //c.clearCachesIfNeeded(deviceData.Ltime)
            self._cache.put(USERAGENT_CACHE_TYPE.to_string(), self.get_user_agent_cache_key(headers.clone()).unwrap(), device.clone());
            return Ok(device);
        } else {
            return Err(device_res.err().unwrap());
        }
    }

    fn clear_caches(&mut self) {
        // This one clears the caches that associates headers to devices and WURFL IDs to devices
        self._cache.clear();

        // the following calls clear frequently used "enumeration fields" which is very time consuming
        // to download every time
        let mk_md_lock_res = self._make_models.lock();
        if mk_md_lock_res.is_ok() {
            let mut make_models = mk_md_lock_res.unwrap();
            make_models.clear();
        }

        let dev_makes_lock_guard = self._device_makes.lock();
        if dev_makes_lock_guard.is_ok() {
            let mut device_makes = dev_makes_lock_guard.unwrap();
            device_makes.clear();
            let dev_makes_map_guard = self._device_makes_map.lock();
            if dev_makes_map_guard.is_ok(){
                dev_makes_map_guard.unwrap().clear();
            }

        }

        let dev_os_lock_res = self._device_oses.lock();
        if dev_os_lock_res.is_ok() {
            let mut device_oses = dev_os_lock_res.unwrap();
            device_oses.clear();
        }

        let os_ver_map_lock_res = self._device_os_versions_map.lock();
        if os_ver_map_lock_res.is_ok() {
            let mut os_ver_map = os_ver_map_lock_res.unwrap();
            os_ver_map.clear();
        }
    }

    /// Sets the new cache size. Changing cache size will result in a cache purge.
    pub fn set_cache_size(&mut self, ua_max_entries: usize) {
        self._cache = Cache::new(ua_max_entries);
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

    /// set_requested_static_capabilities - set list of standard static capabilities to return
    pub fn set_requested_static_capabilities(&mut self, cap_list: Option<Vec<&str>>) {
        if cap_list.is_none() {
            self.requested_static_caps = None;
            self.clear_caches();
            return;
        }

        let mut cap_names: Vec<String> = vec![];
        for name in cap_list.unwrap() {
            if self.has_static_capability(name) {
                cap_names.push(name.to_string());
            }
        }
        if cap_names.len() > 0 {
            self.requested_static_caps = Some(cap_names);
            self.clear_caches();
        }
    }

    /// set_requested_virtual_capabilities - set list of standard virtual capabilities to return
    pub fn set_requested_virtual_capabilities(&mut self, vcap_list: Option<Vec<&str>>) {
        if vcap_list.is_none() {
            self.requested_virtual_caps = None;
            self.clear_caches();
            return;
        }

        let mut virtual_cap_names: Vec<String> = vec![];
        for name in vcap_list.unwrap() {
            if self.has_virtual_capability(name) {
                virtual_cap_names.push(name.to_string());
            }
        }
        if virtual_cap_names.len() > 0 {
            self.requested_virtual_caps = Some(virtual_cap_names);
            self.clear_caches();
        }
    }

    /// SetRequestedCapabilities - set the given capability names to the set they belong
    pub fn set_requested_capabilities(&mut self, cap_list: Option<Vec<&str>>) {
        if cap_list.is_none() {
            self.requested_static_caps = None;
            self.requested_virtual_caps = None;
            self.clear_caches();
            return;
        }

        let mut cap_names: Vec<String> = vec![];
        let mut vcap_names: Vec<String> = vec![];
        for name in cap_list.unwrap() {
            if self.has_static_capability(name) {
                cap_names.push(name.to_string());
            } else if self.has_virtual_capability(name) {
                vcap_names.push(name.to_string());
            }
        }

        self.requested_static_caps = Some(cap_names);
        self.requested_virtual_caps = Some(vcap_names);
        self.clear_caches();
    }

    fn _internal_lookup(&self, request: Request, path: String) -> Result<JSONDeviceData, WmError> {
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
                        return Err(WmError { msg: serde_err.unwrap().to_string() });
                    } else {
                        return Err(WmError { msg: "Unable to parse JSON response".to_string() });
                    }
                }
            } else {
                return Err(WmError { msg: str_resp_res.err().unwrap().to_string() });
            }
        } else {
            return Err(WmError { msg: resp_res.err().unwrap().to_string() });
        }
    }

    fn _clear_caches_if_needed(&mut self, ltime: String) {
        if ltime.len() > 0 && self._ltime != ltime {
            self._ltime = ltime.to_string();
            self.clear_caches();
        }
    }

    /// get_actual_cache_sizes returns the values of cache size. The first value being the device-id based cache, the second value being
    /// the size of the headers-based one
    pub fn get_actual_cache_sizes(&self) -> (usize, usize) {
        return self._cache.get_actual_sizes();
    }

    /// get_all_oses returns a vec<String> of all devices device_os capabilities in WM server
    pub fn get_all_oses(&self) -> Result<Vec<String>, WmError> {
        let os_data = self._load_device_os_data();
        if os_data.is_some() {
            let wm_err = os_data.unwrap();
            return Err(wm_err);
        }

        let os_guard = self._device_oses.lock();
        if os_guard.is_ok() {
            let vec = os_guard.unwrap();
            let ret_val = vec.to_vec();
            return Ok(ret_val);
        } else {
            let guard_err = os_guard.err().unwrap();
            return Err(WmError { msg: format!("Cannot retrieve device OS list: {}", guard_err.to_string()) });
        }
    }

    pub fn get_all_versions_for_os(&self, os_name: &str) -> Result<Vec<String>, WmError> {
        let mut os_versions: Vec<String> = Vec::new();
        let os_data = self._load_device_os_data();
        if os_data.is_some() {
            let wm_err = os_data.unwrap();
            return Err(wm_err);
        }

        let os_ver_map_guard = self._device_os_versions_map.lock();
        if os_ver_map_guard.is_ok() {
            let os_ver_map = os_ver_map_guard.unwrap();
            if os_ver_map.contains_key(os_name) {
                let os_vers_from_map = os_ver_map.get(os_name).unwrap();
                for val in os_vers_from_map {
                    if "" != val.as_str() {
                        os_versions.push(val.to_string());
                    }
                }
                os_versions.sort();
                return Ok(os_versions.clone());
            } else {
                return Err(WmError { msg: format!("Error getting data from WM server: {} does not exist or has no versions", os_name) });
            }
        } else {
            let guard_err = os_ver_map_guard.err().unwrap();
            return Err(WmError { msg: format!("Cannot retrieve device OS versions list: {}", guard_err.to_string()) });
        }
    }

    pub fn get_all_device_makes(&self) -> Result<Vec<String>, WmError> {
        let makes_data = self._load_device_makes_data();
        if makes_data.is_some() {
            let wm_err = makes_data.unwrap();
            return Err(wm_err);
        }

        let guard = self._device_makes.lock();
        if guard.is_ok() {
            let vec = guard.unwrap();
            let ret_val = vec.to_vec();
            return Ok(ret_val);
        } else {
            let guard_err = guard.err().unwrap();
            return Err(WmError { msg: format!("Cannot retrieve device makes list: {}", guard_err.to_string()) });
        }
    }

    pub fn get_all_devices_for_make(&self, brand_name: String) -> Result<Vec<JSONModelMktName>, WmError> {

        let makes_data = self._load_device_makes_data();
        if makes_data.is_some() {
            let wm_err = makes_data.unwrap();
            return Err(wm_err);
        }

        let guard = self._device_makes_map.lock();
        if guard.is_ok(){
            let device_makes_map = guard.unwrap();
            let vec_opt = device_makes_map.get(brand_name.as_str());
            if vec_opt.is_some(){
                let mut ret_vec: Vec<JSONModelMktName> = Vec::new();
                let md_mk_vec = vec_opt.unwrap();
                for md_mk in md_mk_vec {
                    let md_mk_copy = JSONModelMktName {
                        model_name: md_mk.model_name.to_string(),
                        marketing_name: md_mk.marketing_name.to_string(),
                    };
                    ret_vec.push(md_mk_copy);
                }
                return Ok(ret_vec);

            } else {
                return Err(WmError { msg: format!("Error getting data from WM server: brand {} does not exist or has no devices", brand_name) });
            }
        }
        else {
            let guard_err = guard.err();
            return Err(WmError { msg: format!("Error getting data from WM server: {}", guard_err.unwrap().to_string()) });
        }

    }

    fn _load_device_os_data(&self) -> Option<WmError> {
        let os_guard = self._device_oses.lock();
        if os_guard.is_ok() {
            let os_vec = os_guard.unwrap();
            if !os_vec.is_empty() {
                return None;
            }
        }
        // TODO: handle case in which os_guard is NOT ok

        // this struct is a vector holding pairs of os name ("Android") and version ("10.0")
        let mut os_version_pairs: Vec<JSONDeviceOsVersions> = Vec::with_capacity(1000);

        let result = self.internal_get("/v2/alldeviceosversions/json");
        match result {
            Ok(res) => {
                let res_string = res.into_string();
                if res_string.is_ok() {
                    let os_vers_str = res_string.unwrap();
                    let _res: Result<Vec<JSONDeviceOsVersions>, serde_json::Error> = serde_json::from_str(os_vers_str.as_str());
                    if _res.is_ok() {
                        os_version_pairs = _res.unwrap();
                    }
                }
            } // If we are here, something went wrong during download
            Err(wm_err) => {
                return Some(wm_err);
            }
        }

        // If we are here, data download succeeded, now let's create the data structure that we'll use to return OS and version enumerations.
        // this is a map that binds each OS name to a vector of their versions
        let mut ov_map: HashMap<String, Vec<String>> = HashMap::new();

        // we'll now check if an OS name has already been added to this map,
        // if not, we'll create a vector to hold the OS versions and add it to the map together with its OS name as key.
        // If OS name exists in the map, we just get the vector to which is associated and just add the os version value to the vector.
        // Version number are guaranteed to be unique for each OS name.
        for ov_item in os_version_pairs {
            if !ov_map.contains_key(ov_item.device_os.as_str()) {
                let mut ov: Vec<String> = Vec::new();
                ov.push(ov_item.device_os_version.clone());
                ov_map.insert(ov_item.device_os, ov.clone());
            } else {
                // we need to the get the vec as mutable to add an item to it
                let ov_vec_opt = ov_map.get_mut(ov_item.device_os.as_str());
                let ov_vec = ov_vec_opt.unwrap();
                ov_vec.push(ov_item.device_os_version);
            }
        }

        // we now use the keys of the map (all OSes) to fill the OSes vector
        let dev_os_guard = self._device_oses.lock();
        let mut os_vec = dev_os_guard.unwrap();
        os_vec.clear();
        let keys = ov_map.keys();
        for k in keys {
            os_vec.push(k.to_string());
        }

        // fill the wm client field with the results of the previous process
        let dev_os_ver_map_guard = self._device_os_versions_map.lock();
        let mut dev_os_ver_map = dev_os_ver_map_guard.unwrap();
        dev_os_ver_map.clear();
        dev_os_ver_map.extend(ov_map);
        return None;
    }

    fn internal_get(&self, path: &str) -> Result<Response, WmError> {
        let full_url = self.create_url(path);
        match ureq::get(full_url.as_str()).set("content-type", DEFAULT_CONTENT_TYPE)
            .call() {
            Ok(res) => {
                return Ok(res);
            }
            Err(i_err) => {
                return Err(WmError { msg: format!(" Unable to get data from {}: {}", full_url, i_err.to_string()) });
            }
        };
    }

    fn _load_device_makes_data(&self) -> Option<WmError> {
        // We lock the shared makeModel cache
        let dev_makes_guard = self._device_makes.lock();
        if !dev_makes_guard.is_ok() {
            let err = dev_makes_guard.err().unwrap();
            return Some(WmError { msg: format!("Cannot download device makes data: {}", err.to_string()) });
        } else {
            let dev_makes = dev_makes_guard.unwrap();
            if dev_makes.len() > 0 {
                // cache has already been loaded or refreshed, return None
                return None;
            }
        }

        let mut mk_models: Vec<JSONMakeModel> = Vec::with_capacity(1000);
        let all_devices_res = self.internal_get("/v2/alldevices/json");
        match all_devices_res {
            Ok(res) => {
                let res_string = res.into_string();
                if res_string.is_ok() {
                    let _res: Result<Vec<JSONMakeModel>, serde_json::Error> = serde_json::from_str(res_string.unwrap().as_str());
                    if _res.is_ok() {
                        mk_models = _res.unwrap();
                    } else {
                        return Some(WmError { msg: format!("Could not parse device makes data {} ", _res.err().unwrap().to_string()) });
                    }
                } else {
                    let err = res_string.err();
                    return Some(WmError { msg: format!("Could not parse device makes data {} ", err.unwrap().to_string()) });
                }
            }
            Err(wm_err) => {
                return Some(wm_err);
            }
        }

        let mut dev_makes_map: HashMap<String, Vec<JSONModelMktName>> = HashMap::new();
        for make_model in mk_models {
            let mut marketing_name = "".to_string();
            if make_model.marketing_name.is_some(){
                marketing_name = make_model.marketing_name.unwrap();
            }


            let md_mk_name = JSONModelMktName {
                model_name: make_model.model_name.to_string(),
                marketing_name,
            };
            if !dev_makes_map.contains_key(make_model.brand_name.as_str()) {
                let mut model_market_names: Vec<JSONModelMktName> = Vec::new();
                model_market_names.push(md_mk_name);
                dev_makes_map.insert(make_model.brand_name, model_market_names);
            } else {
                let model_market_names_opt = dev_makes_map.get_mut(make_model.brand_name.as_str());
                let model_market_vec = model_market_names_opt.unwrap();
                model_market_vec.push(md_mk_name);
            }
        }
        let dev_makes_guard = self._device_makes.lock();
        let mut dev_makes_vec = dev_makes_guard.unwrap();
        dev_makes_vec.clear();
        let keys = dev_makes_map.keys();
        for k in keys {
            dev_makes_vec.push(k.to_string());
        }
        // fill the wm client field with the results of the previous process
        let dev_make_model_map_guard = self._device_makes_map.lock();
        // We can unwrap safely since we know we created it.
        let mut dev_make_model_map = dev_make_model_map_guard.unwrap();
        dev_make_model_map.clear();
        dev_make_model_map.extend(dev_makes_map);
        return None;
    }
}
