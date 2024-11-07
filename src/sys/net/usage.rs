use crate::api::fs::{FileIO, IO};
use crate::sys::net::EthernetDeviceIO;

use alloc::format;

#[derive(Debug, Clone)]
pub struct NetUsage;

impl NetUsage {
    pub fn new() -> Self {
        Self
    }

    pub fn size() -> usize {
        83
    }
}

impl FileIO for NetUsage {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        if let Some((_, ref mut device)) = *super::NET.lock() {
            let stats = device.stats();
            let s = format!(
                "{} {} {} {}",
                stats.rx_packets_count(),
                stats.rx_bytes_count(),
                stats.tx_packets_count(),
                stats.tx_bytes_count(),
            );
            let n = s.len();
            buf[0..n].copy_from_slice(s.as_bytes());
            return Ok(n);
        }
        Err(())
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

