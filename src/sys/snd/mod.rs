mod sb16;

use crate::api::fs::{FileIO, IO};

#[derive(Debug, Clone)]
pub struct SndBuffer;

impl SndBuffer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn size() -> usize {
        sb16::BUF_LEN
    }
}

impl FileIO for SndBuffer {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ()> {
        Err(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        if buf.is_empty() {
            sb16::stop();
        } else {
            sb16::play(buf);
        }
        Ok(buf.len())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => false,
            IO::Write => true,
        }
    }
}

pub fn init() {
    sb16::init();
}
