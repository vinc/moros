#[cfg(target_arch = "x86_64")]
use crate::sys::fs::{FileIO, IO};

#[derive(Debug, Clone)]
pub struct Buffer;

impl Buffer {
    pub fn new() -> Self {
        Self
    }

    pub const fn addr() -> usize {
        0xA0000
    }

    pub const fn size() -> usize {
        // TODO: The buffer size is dependent on the current VGA mode so this
        // should be init to 1 and the size in the dir entry might be updated
        // when a new mode is set.
        320 * 200
    }
}

#[cfg(target_arch = "x86_64")]
impl FileIO for Buffer {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, ()> {
        Err(()) // TODO
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        // FIXME: This only work for 320x200 linear buffer
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
