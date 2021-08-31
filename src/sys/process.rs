use crate::sys::fs::File;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

const MAX_FILE_HANDLES: usize = 1024;

lazy_static! {
    pub static ref PIDS: AtomicUsize = AtomicUsize::new(0);
    pub static ref PROCESS: Mutex<Process> = Mutex::new(Process::new("/", None)); // TODO
}

pub struct Process {
    id: usize,
    env: BTreeMap<String, String>,
    dir: String,
    user: Option<String>,
    file_handles: Vec<Option<File>>,
}

impl Process {
    pub fn new(dir: &str, user: Option<&str>) -> Self {
        let id = PIDS.fetch_add(1, Ordering::SeqCst);
        let env = BTreeMap::new();
        let dir = dir.to_string();
        let user = user.map(String::from);
        let file_handles = vec![None; MAX_FILE_HANDLES];
        Self { id, env, dir, user, file_handles }
    }
}

pub fn id() -> usize {
    PROCESS.lock().id
}

pub fn env(key: &str) -> Option<String> {
    PROCESS.lock().env.get(key).cloned()
}

pub fn envs() -> BTreeMap<String, String> {
    PROCESS.lock().env.clone()
}

pub fn dir() -> String {
    PROCESS.lock().dir.clone()
}

pub fn user() -> Option<String> {
    PROCESS.lock().user.clone()
}

pub fn set_env(key: &str, val: &str) {
    PROCESS.lock().env.insert(key.into(), val.into());
}

pub fn set_dir(dir: &str) {
    PROCESS.lock().dir = dir.into();
}

pub fn set_user(user: &str) {
    PROCESS.lock().user = Some(user.into())
}

pub fn create_file_handle(file: File) -> Result<usize, ()> {
    let min = 4; // The first 4 file handles are reserved
    let max = MAX_FILE_HANDLES;
    let proc = &mut *PROCESS.lock();
    for fh in min..max {
        if proc.file_handles[fh].is_none() {
            proc.file_handles[fh] = Some(file);
            return Ok(fh);
        }
    }
    Err(())
}

pub fn update_file_handle(fh: usize, file: File) {
    let proc = &mut *PROCESS.lock();
    proc.file_handles[fh] = Some(file);
}

pub fn delete_file_handle(fh: usize) {
    let proc = &mut *PROCESS.lock();
    proc.file_handles[fh] = None;
}

pub fn file_handle(fh: usize) -> Option<File> {
    let proc = &mut *PROCESS.lock();
    proc.file_handles[fh].clone()
}
