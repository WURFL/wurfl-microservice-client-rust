use std::collections::HashMap;
use wmclient::{Cache, DEVICE_ID_CACHE_TYPE, JSONDeviceData, USERAGENT_CACHE_TYPE};

#[test]
fn create_empty_cache_and_get_test(){
    let cache = Cache::new(100);
    let sizes = cache.get_actual_sizes();
    // Nothing inside
    assert_eq!(0, sizes.0);
    assert_eq!(0, sizes.1);

    // 1 - We try to get something from an empty cache and for an empty cache type: we get NONE
    let dev = cache.get("".to_string(), "key".to_string());
    assert!(dev.is_none());

    // 2 - We try to get something from an empty cache and for an actual cache type: we get NONE again
    let dev = cache.get(DEVICE_ID_CACHE_TYPE.to_string(), "key".to_string());
    assert!(dev.is_none())
}

#[test]
fn create_put_and_get_test(){
    let cache = Cache::new(100);

    let device = JSONDeviceData{
        capabilities: HashMap::new(),
        ltime: "1234567989".to_string(),
        error: "".to_string(),
        mtime: 123465879
    };
    cache.put(USERAGENT_CACHE_TYPE.to_string(), "test".to_string(), device);
    let sizes = cache.get_actual_sizes();
    // We have put one element in the headers based cache
    assert_eq!(0, sizes.0);
    assert_eq!(1, sizes.1);

    // We try to get the device for the proper cache type
    let dev = cache.get(USERAGENT_CACHE_TYPE.to_string(), "test".to_string());
    assert!(dev.is_some());
    let get_device = dev.unwrap();
    assert_eq!("1234567989", get_device.ltime);
    assert_eq!("", get_device.error);
    assert_eq!(123465879, get_device.mtime);
    // Device is on the partition for header based cache, so it is not found if get with other cache types
    let none_dev = cache.get(DEVICE_ID_CACHE_TYPE.to_string(), "test".to_string());
    assert!(none_dev.is_none());
}

#[test]
fn clear_test(){
    let cache = Cache::new(100);

    let device = JSONDeviceData{
        capabilities: HashMap::new(),
        ltime: "1234567989".to_string(),
        error: "".to_string(),
        mtime: 123465879
    };

    let device2 = JSONDeviceData{
        capabilities: HashMap::new(),
        ltime: "1234587989".to_string(),
        error: "".to_string(),
        mtime: 123465679
    };

    cache.put(USERAGENT_CACHE_TYPE.to_string(), "test".to_string(), device);
    cache.put(DEVICE_ID_CACHE_TYPE.to_string(), "test".to_string(), device2);
    let sizes = cache.get_actual_sizes();
    assert_eq!(1, sizes.1);
    assert_eq!(1, sizes.1);

    cache.clear();
    // This must be empty now
    let sizes = cache.get_actual_sizes();
    assert_eq!(0, sizes.1);
    assert_eq!(0, sizes.1);
}