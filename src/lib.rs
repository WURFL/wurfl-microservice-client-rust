use lru::LruCache;
use std::collections::HashMap;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use ureq::{Agent, Response};
include!("./wmclient.rs");
include!("./model.rs");
include!("./cache.rs");