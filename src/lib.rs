use lru::LruCache;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use md5;
use ureq::Response;
include!("./wmclient.rs");
include!("./model.rs");
include!("./cache.rs");