use crate::api::fs::{FileIO, IO};

use alloc::format;
use alloc::string::String;
use core::str::FromStr;
use smoltcp::wire::IpCidr;

#[derive(Debug, Clone)]
pub struct NetIp;

impl NetIp {
    pub fn new() -> Self {
        Self
    }

    pub fn size() -> usize {
        16 + 1 + 3
    }
}

impl FileIO for NetIp {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        if let Some((ref mut iface, _)) = *super::NET.lock() {
            if let Some(ip) = iface.ip_addrs().iter().next() {
                let s = format!("{}/{}", ip.address(), ip.prefix_len());
                let n = s.len();
                buf[0..n].copy_from_slice(s.as_bytes());
                return Ok(n);
            }
        }
        Err(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        if let Ok(s) = String::from_utf8(buf.to_vec()) {
            if let Ok(addr) = IpCidr::from_str(&s) {
                if let Some((ref mut iface, _)) = *super::NET.lock() {
                    iface.update_ip_addrs(|addrs| {
                        addrs.clear();
                        addrs.push(addr).unwrap();
                        log!("NET IP {}", s);
                    });
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

