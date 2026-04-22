use crate::api::fs::{FileIO, IO};

use alloc::string::String;

#[derive(Debug, Clone)]
pub struct ProcUser;

impl ProcUser {
    pub fn new() -> Self {
        Self
    }

    pub fn size() -> usize {
        16
    }
}

impl FileIO for ProcUser {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let s = super::user().unwrap_or(String::new());
        let n = s.len();
        if n > buf.len() {
            return Err(());
        }
        buf[0..n].copy_from_slice(s.as_bytes());
        Ok(n)
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, ()> {
        Err(())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => false,
        }
    }
}
