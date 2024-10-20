use crate::api::fs::{FileIO, IO};

#[derive(Debug, Clone)]
pub struct Uptime;

impl Uptime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn size() -> usize {
        core::mem::size_of::<f64>()
    }
}

impl FileIO for Uptime {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let time = uptime().to_be_bytes();
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

// NOTE: This clock is monotonic
pub fn uptime() -> f64 {
    super::time_between_ticks() * super::ticks() as f64
}

#[test_case]
fn test_uptime() {
    assert!(uptime() > 0.0);
}
