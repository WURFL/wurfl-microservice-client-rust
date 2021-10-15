pub struct Cache {
    _ua_cache: RwLock<LruCache<String, JSONDeviceData>>,
    _dev_id_cache: RwLock<LruCache<String, JSONDeviceData>>,
}

impl Cache {
    pub fn new(max_size: usize) -> Cache {
        Cache {
            _ua_cache: RwLock::new(LruCache::new(max_size)),
            _dev_id_cache: RwLock::new(LruCache::new(20000)),
        }
    }

    pub fn clear(&self) {
        {
            let mut l_ua_cache = self._ua_cache.write().unwrap();
            l_ua_cache.clear();
        }

        {
            let mut l_did_cache = self._dev_id_cache.write().unwrap();
            l_did_cache.clear();
        }
    }

    pub fn get_actual_sizes(&self) -> (usize, usize) {
        let ua_guard = self._ua_cache.read().unwrap();
        let ua_size = ua_guard.len();
        drop(ua_guard);

        let did_guard = self._dev_id_cache.read().unwrap();
        let did_size = did_guard.len();
        drop(did_guard);

        (did_size, ua_size)
    }

    pub fn put(&self, cache_type: String, key: String, value: JSONDeviceData) {
        if cache_type == USERAGENT_CACHE_TYPE {
            _internal_put(&self._ua_cache, key, value);
        } else if cache_type == DEVICE_ID_CACHE_TYPE {
            _internal_put(&self._dev_id_cache, key, value);
        }
    }

    pub fn get(&self, cache_type: String, key: String) -> Option<JSONDeviceData> {
        if cache_type == USERAGENT_CACHE_TYPE {
            return _internal_get(&self._ua_cache, key);
        } else if cache_type == DEVICE_ID_CACHE_TYPE {
             return _internal_get(&self._dev_id_cache, key);
        }
        None
    }
}

fn _internal_get(cache: &RwLock<LruCache<String, JSONDeviceData>>, key: String) -> Option<JSONDeviceData> {
    let mut cache_guard = cache.write().unwrap();

        let opt = cache_guard.get(&key);
        if opt.is_some() {
            let d_ref = opt.unwrap();
            let device = d_ref.clone();
            return Some(device);
        }
        return None;
}

fn _internal_put(cache: &RwLock<LruCache<String, JSONDeviceData>>, key: String, value: JSONDeviceData) {
    let mut cache_guard = cache.write().unwrap();
    cache_guard.put(key, value);
}

