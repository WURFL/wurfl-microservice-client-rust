use lru::LruCache;
use std::collections::HashMap;
use ureq::{Response, Error};
use std::sync::Mutex;
include!("./wmclient.rs");
include!("./model.rs");