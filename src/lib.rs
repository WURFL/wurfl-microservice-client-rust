use lru::LruCache;
use std::collections::HashMap;
use ureq::{Response, Error};
use std::sync::Mutex;
use md5;
use thiserror::private::AsDynError;
include!("./wmclient.rs");
include!("./model.rs");