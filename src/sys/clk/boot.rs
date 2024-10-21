use super::timer;

use crate::api::fs::{FileIO, IO};

#[derive(Debug, Clone)]
pub struct BootTime;

impl BootTime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn size() -> usize {
        core::mem::size_of::<f64>()
    }
}

impl FileIO for BootTime {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let time = boot_time().to_be_bytes();
        let n = time.len();
        if buf.len() >= n {
            buf[0..n].clone_from_slice(&time);
            Ok(n)
        } else {
            Err(())
        }
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, ()> {
        unimplemented!();
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => false,
        }
    }
}

/// Returns the number of seconds since boot.
///
/// This clock is monotonic.
pub fn boot_time() -> f64 {
    timer::time_between_ticks() * timer::ticks() as f64
}

#[test_case]
fn test_boot_time() {
    assert!(boot_time() > 0.0);
}
