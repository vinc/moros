use crate::api::syscall;
use crate::sys::fs::FileType;

use alloc::string::{String, ToString};
use alloc::vec;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self {}
    }

    pub fn read_char(&self) -> Option<char> {
        let mut buf = vec![0; 4];
        if let Some(bytes) = syscall::read(0, &mut buf) {
            if bytes > 0 {
                buf.resize(bytes, 0);
                let s = String::from_utf8_lossy(&buf).to_string().remove(0);
                return Some(s);
            }
        }
        None
    }

    pub fn read_line(&self) -> String {
        let mut buf = vec![0; 256];
        if let Some(bytes) = syscall::read(0, &mut buf) {
            buf.resize(bytes, 0);
            String::from_utf8_lossy(&buf).to_string()
        } else {
            String::new()
        }
    }
}

impl Stdout {
    fn new() -> Self {
        Self {}
    }

    pub fn write(&self, s: &str) {
        syscall::write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self {}
    }

    pub fn write(&self, s: &str) {
        syscall::write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}

pub fn is_redirected(handle: usize) -> bool {
    match syscall::kind(handle) {
        Some(FileType::File) => true,
        _ => false,
    }
}
