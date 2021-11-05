use crate::api::syscall;

use alloc::vec;
use alloc::string::{String, ToString};

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self {}
    }

    pub fn read_char(&self) -> Option<char> {
        let mut buf = vec![0; 1];
        if let Some(bytes) = syscall::read(0, &mut buf) {
            if bytes > 0 {
                return Some(buf[0] as char);
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
