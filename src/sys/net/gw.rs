use crate::api::fs::{FileIO, IO};

use alloc::string::{String, ToString};
use core::str::FromStr;
use smoltcp::wire::Ipv4Address;

#[derive(Debug, Clone)]
pub struct NetGw;

impl NetGw {
    pub fn new() -> Self {
        Self
    }

    pub fn size() -> usize {
        16
    }
}

impl FileIO for NetGw {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        if let Some((ref mut iface, _)) = *super::NET.lock() {
            let mut n = 0;
            iface.routes_mut().update(|storage| {
                if let Some(route) = storage.iter().next() {
                    let s = route.via_router.to_string();
                    n = s.len();
                    buf[0..n].copy_from_slice(s.as_bytes());
                }
            });
            if n > 0 {
                return Ok(n);
            }
        }
        Err(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        if let Some((ref mut iface, _)) = *super::NET.lock() {
            if let Ok(s) = String::from_utf8(buf.to_vec()) {
                if s == "0.0.0.0" {
                    iface.routes_mut().remove_default_ipv4_route();
                    return Ok(s.len());
                } else if let Ok(ip) = Ipv4Address::from_str(&s) {
                    iface.routes_mut().add_default_ipv4_route(ip).unwrap();
                    log!("NET GW {}", s);
                    return Ok(s.len());
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

