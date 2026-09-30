use crate::sys::net::{Config, EthernetDeviceIO, Stats};
use crate::sys::x86::port::*;

use alloc::sync::Arc;
use alloc::vec::Vec;

const MTU: usize = 1536;

#[derive(Clone)]
pub struct Device {
    io_base: u16,
    config: Arc<Config>,
    stats: Arc<Stats>,
    tx_buffer: Vec<u8>,
}

impl Device {
    pub fn new(io_base: u16) -> Self {
        let mut device = Self {
            io_base,
            config: Arc::new(Config::new()),
            stats: Arc::new(Stats::new()),
            tx_buffer: Vec::with_capacity(MTU),
        };
        device.init();
        device
    }

    fn init(&mut self) {
    }
}

impl EthernetDeviceIO for Device {
    fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    fn stats(&self) -> Arc<Stats> {
        self.stats.clone()
    }

    fn receive_packet(&mut self) -> Option<Vec<u8>> {
        return None;
    }

    fn transmit_packet(&mut self, len: usize) {
    }

    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8] {
        &mut self.tx_buffer[0..len]
    }
}
