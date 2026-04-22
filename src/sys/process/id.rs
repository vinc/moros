use crate::api::fs::{FileIO, IO};

use alloc::format;

#[derive(Debug, Clone)]
pub struct ProcId;

impl ProcId {
    pub fn new() -> Self {
        Self
    }

    pub fn size() -> usize {
        10 // Must be greater than 8 to be considered as a block device
    }
}

impl FileIO for ProcId {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let s = format!("{}", super::id());
        let n = s.len();
        buf[0..n].copy_from_slice(s.as_bytes());
        if n > buf.len() {
            return Err(());
        }
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
