pub struct Cache {
    _ua_cache: Arc<Mutex<LruCache<String, JSONDeviceData>>>,
    _dev_id_cache: Arc<Mutex<LruCache<String, JSONDeviceData>>>,
}

impl Cache {
    pub fn new(max_size: usize) -> Cache {
        Cache {
            _ua_cache: Arc::new(Mutex::new(LruCache::new(max_size))),
            _dev_id_cache: Arc::new(Mutex::new(LruCache::new(20000))),
        }
    }

    pub fn clear(&self) {
        // weird: seems that mutex guard must be mutable if its content is mutable.
        let mut guard = self._ua_cache.lock().unwrap();
        guard.clear();
        drop(guard);

        let mut did_guard = self._dev_id_cache.lock().unwrap();
        did_guard.clear();
        drop(did_guard);
    }

    pub fn get_actual_sizes(&self) -> (usize, usize) {
        let ua_guard = self._ua_cache.lock().unwrap();
        let ua_size = ua_guard.len();
        drop(ua_guard);

        let did_guard = self._dev_id_cache.lock().unwrap();
        let did_size = did_guard.len();
        drop(did_guard);

        return (did_size, ua_size);
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
             _internal_get(&self._ua_cache, key);
        } else if cache_type == DEVICE_ID_CACHE_TYPE {
             _internal_get(&self._dev_id_cache, key);
        }
        return None;
    }
}

fn _internal_get(cache: &Arc<Mutex<LruCache<String, JSONDeviceData>>>, key: String) -> Option<JSONDeviceData> {
    let mut cache_guard = cache.lock().unwrap();

        let opt = cache_guard.get(&key);
        if opt.is_some() {
            let d_ref = opt.unwrap();
            let device = d_ref.clone();
            return Some(device);
        }
        return None;
}

fn _internal_put(cache: &Arc<Mutex<LruCache<String, JSONDeviceData>>>, key: String, value: JSONDeviceData) {
    let mut cache_guard = cache.lock().unwrap();
    cache_guard.put(key, value);
}

