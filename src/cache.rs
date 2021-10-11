pub struct Cache {
    _ua_cache: Arc<Mutex<Option<LruCache<String, JSONDeviceData>>>>,
    _dev_id_cache: Arc<Mutex<Option<LruCache<String, JSONDeviceData>>>>,
}

impl Cache {

    pub fn new(max_size: usize) -> Cache{
        if max_size > 0 {
            return Cache{
                _ua_cache: Arc::new(Mutex::new(Some(LruCache::new(max_size)))),
                _dev_id_cache: Arc::new(Mutex::new(Some(LruCache::new(20000))))
            }
        } else {
            return Cache{
                _ua_cache: Arc::new(Mutex::new(None)),
                _dev_id_cache: Arc::new(Mutex::new(None))
            }
        }
    }

    pub fn clear(&self) {
        // weird: seems that mutex guard must be mutable if its content is mutable.
        let mut guard = self._ua_cache.lock().unwrap();
        if guard.is_some() {
            guard.as_mut().unwrap().clear();
        }
        drop(guard);

        let mut did_guard = self._dev_id_cache.lock().unwrap();
        if did_guard.is_some() {
            did_guard.as_mut().unwrap().clear();
        }
        drop(did_guard);
    }

    pub fn get_actual_sizes(&self) -> (usize, usize) {
        let mut ua_size: usize = 0;
        let mut did_size: usize = 0;
        let mut ua_guard = self._ua_cache.lock().unwrap();
        if ua_guard.is_some() {
            ua_size = ua_guard.as_mut().unwrap().len();
        }

        drop(ua_guard);

        let mut did_guard = self._dev_id_cache.lock().unwrap();
        if did_guard.is_some() {
            did_size = did_guard.as_mut().unwrap().len();
        }
        drop(did_guard);

        return (did_size, ua_size);
    }

    pub fn put(&self, cache_type: String, key: String, value: JSONDeviceData){
        if cache_type == USERAGENT_CACHE_TYPE {
            _internal_put(&self._ua_cache, key, value);
        }
        else if cache_type == DEVICE_ID_CACHE_TYPE {
            _internal_put(&self._dev_id_cache, key, value);
        }
    }

    pub fn get(&self, cache_type: String, key: String) -> Option<JSONDeviceData> {
        if cache_type == USERAGENT_CACHE_TYPE {
            return _internal_get(&self._ua_cache, key);
        }
        else if cache_type == DEVICE_ID_CACHE_TYPE {
            return _internal_get(&self._dev_id_cache, key);
        }
        return None;
    }
}

fn _internal_get(cache: &Arc<Mutex<Option<LruCache<String, JSONDeviceData>>>>, key: String) -> Option<JSONDeviceData> {
    let mut cache_guard = cache.lock().unwrap();
    if cache_guard.is_some(){
        let opt = cache_guard.as_mut().unwrap().get(&key);
        if opt.is_some(){
            let d_ref = opt.unwrap();
            let device = JSONDeviceData{
                capabilities: d_ref.capabilities.clone(),
                error: d_ref.error.clone(),
                mtime: d_ref.mtime.clone(),
                ltime: d_ref.ltime.clone()
            };
            return Some(device);
        }
        return None;
    }
    drop(cache_guard);
    return None;
}

fn _internal_put(cache: &Arc<Mutex<Option<LruCache<String, JSONDeviceData>>>>, key: String, value: JSONDeviceData) {
    let mut cache_guard = cache.lock().unwrap();
    if cache_guard.is_some(){
        cache_guard.as_mut().unwrap().put(key, value);
    }
}

