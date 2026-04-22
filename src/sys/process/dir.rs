use crate::api::fs;
use crate::api::fs::{FileIO, IO};
use crate::sys::fs::Dir;

use alloc::string::String;

#[derive(Debug, Clone)]
pub struct ProcDir;

impl ProcDir {
    pub fn new() -> Self {
        Self
    }

    pub fn size() -> usize {
        256
    }
}

impl FileIO for ProcDir {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let s = super::dir();
        let n = s.len();
        if n > buf.len() {
            return Err(());
        }
        buf[0..n].copy_from_slice(s.as_bytes());
        Ok(n)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        if let Ok(s) = String::from_utf8(buf.to_vec()) {
            let s = s.trim();
            if !s.is_empty() {
                let mut path = fs::realpath(&s);
                if path.len() > 1 {
                    path = path.trim_end_matches('/').into();
                }
                if Dir::open(&path).is_some() {
                    super::set_dir(&path);
                    return Ok(buf.len());
                }
            }
        }
        Err(())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => true,
        }
    }
}
