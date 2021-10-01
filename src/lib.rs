use lru::LruCache;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use md5;
use ureq::Response;
include!("./wmclient.rs");
include!("./model.rs");
include!("./cache.rs");