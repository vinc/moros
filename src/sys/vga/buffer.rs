use crate::api::fs::{FileIO, IO};

#[derive(Debug, Clone)]
pub struct Buffer;

impl Buffer {
    pub fn new() -> Self {
        Self
    }

    pub const fn addr() -> u64 {
        0xA0000
    }

    pub const fn size() -> usize {
        640 * 480 // Max buffer size
    }
}

impl FileIO for Buffer {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ()> {
        Err(()) // TODO
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        let len = buf.len();
        let src = buf.as_ptr();
        let dst = Self::addr() as *mut u8;
        if Self::size() < len {
            return Err(());
        }
        unsafe {
            core::ptr::copy_nonoverlapping(src, dst, len);
        }
        Ok(len)
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => false, // TODO
            IO::Write => true,
        }
    }
}
