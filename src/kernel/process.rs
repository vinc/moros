use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref PIDS: AtomicUsize = AtomicUsize::new(0);
    pub static ref PROCESS: Mutex<Process> = Mutex::new(Process::new("/", "admin")); // TODO
}

pub struct Process {
    id: usize,
    env: BTreeMap<String, String>,
    dir: String,
    user: String, // TODO: Use uid
}

impl Process {
    pub fn new(dir: &str, user: &str) -> Self {
        let id = PIDS.fetch_add(1, Ordering::SeqCst);
        let env = BTreeMap::new();
        let dir = dir.to_string();
        let user = user.to_string();
        Self { id, env, dir, user }
    }
}

pub fn id() -> usize {
    PROCESS.lock().id
}

pub fn env(key: &str) -> Option<String> {
    match PROCESS.lock().env.get(key.into()) {
        Some(val) => Some(val.clone()),
        None => None,
    }
}

pub fn dir() -> String {
    PROCESS.lock().dir.clone()
}

pub fn user() -> String {
    PROCESS.lock().user.clone()
}

pub fn set_env(key: &str, val: &str) {
    PROCESS.lock().env.insert(key.into(), val.into());
}

pub fn set_dir(dir: &str) {
    PROCESS.lock().dir = dir.into();
}

pub fn set_user(user: &str) {
    PROCESS.lock().user = user.into();
}
