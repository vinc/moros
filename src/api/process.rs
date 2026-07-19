use crate::api::fs;
use crate::api::ini;
use crate::api::syscall;

use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    Success        =   0,
    Failure        =   1,
    UsageError     =  64,
    DataError      =  65,
    OpenError      = 128,
    ReadError      = 129,
    ExecError      = 130,
    PanicError     = 200,
    PageFaultError = 201,
    ShellExit      = 255,
}

impl From<usize> for ExitCode {
    fn from(code: usize) -> Self {
        match code {
              0 => ExitCode::Success,
             64 => ExitCode::UsageError,
             65 => ExitCode::DataError,
            128 => ExitCode::OpenError,
            129 => ExitCode::ReadError,
            130 => ExitCode::ExecError,
            200 => ExitCode::PanicError,
            201 => ExitCode::PageFaultError,
            255 => ExitCode::ShellExit,
              _ => ExitCode::Failure,
        }
    }
}

pub fn spawn(path: &str, args: &[&str]) -> Result<(), ExitCode> {
    if syscall::info(path).is_some() {
        match syscall::spawn(path, args) {
            ExitCode::Success => Ok(()),
            code => Err(code),
        }
    } else {
        Err(ExitCode::OpenError)
    }
}

// TODO: Return Result<usize>
pub fn id() -> usize {
    let s = fs::read_to_string("/dev/proc/id").unwrap_or("0".to_string());
    s.parse().unwrap_or(0)
}

// TODO: Return Result<String>
pub fn dir() -> String {
    fs::read_to_string("/dev/proc/dir").unwrap_or("/".to_string())
}

// TODO: Return Result<()>
pub fn set_dir(path: &str) {
    let _ = fs::write("/dev/proc/dir", path.as_bytes());
}

// TODO: Return Result<String>
pub fn user() -> Option<String> {
    fs::read_to_string("/dev/proc/user").ok().filter(|user| !user.is_empty())
}

// TODO: Return Result<BTreeMap<String, String>>
pub fn env() -> BTreeMap<String, String> {
    if let Ok(s) = fs::read_to_string("/dev/proc/env") {
        if let Some(h) = ini::parse(&s) {
            return h;
        }
    }
    BTreeMap::new()
}

pub fn env_var(key: &str) -> Option<String> {
    env().get(key).cloned()
}

// TODO: Return Result<()>
pub fn set_env_var(key: &str, val: &str) {
    let s = format!("{}=\"{}\"", key, val);
    let _ = fs::write("/dev/proc/env", s.as_bytes());
}
