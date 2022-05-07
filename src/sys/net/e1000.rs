use crate::sys;
use crate::sys::allocator::PhysBuf;
use crate::sys::net::{EthernetDeviceIO, Config, Stats};
use crate::sys::pci::DeviceConfig;

use alloc::sync::Arc;
use alloc::vec::Vec;
use bit_field::BitField;
use core::sync::atomic::{AtomicUsize, Ordering};
use smoltcp::wire::EthernetAddress;
use x86_64::instructions::port::Port;

const CTRL:   u16 = 0x0000;
const EEPROM: u16 = 0x0014;
//const EECD:   u16 = 0x00010;

//const CTRL_RST: u32 = 0x4000000;
//const EECD_SK: u32 = 0x01;
//const EECD_CS: u32 = 0x02;
//const EECD_DI: u32 = 0x03;
//const EECD_REQ: u32 = 0x40;
//const EECD_GNT: u32 = 0x80;

const IO_ADDR: u16 = 0x00;
const IO_DATA: u16 = 0x04;

#[derive(Clone)]
pub struct Device {
    io_base: u16,
    bar_type: u16,
    has_eeprom: bool,
    config: Arc<Config>,
    stats: Arc<Stats>,
    rx_buffer: PhysBuf,
    tx_buffer: PhysBuf,
}

impl Device {
    pub fn new(pci: DeviceConfig) -> Self {
        pci.enable_bus_mastering();
        let mut device = Self {
            io_base: pci.io_base(),
            bar_type: pci.bar_type(),
            has_eeprom: false,
            config: Arc::new(Config::new()),
            stats: Arc::new(Stats::new()),
            rx_buffer: PhysBuf::new(1500),
            tx_buffer: PhysBuf::new(1500),
        };
        device.init();
        device
    }

    fn init(&mut self) {
        //self.write_long(CTRL, CTRL_RST);
        //self.config.update_mac(EthernetAddress::from_bytes(&[0, 0, 0, 0, 0, 0]));
        self.detect_eeprom();
        debug!("EEPROM: {}", self.has_eeprom);
        debug!("BAR type: {}", self.bar_type);
        debug!("IO base: {}", self.io_base);
        self.config.update_mac(self.read_mac());
    }

    fn read_mac(&self) -> EthernetAddress {
        let mut mac = [0; 6];
    	let mut tmp;
        tmp = self.read_eeprom(0);
        mac[0] = (tmp &0xff) as u8;
        mac[1] = (tmp >> 8) as u8;
        tmp = self.read_eeprom(1);
        mac[2] = (tmp &0xff) as u8;
        mac[3] = (tmp >> 8) as u8;
        tmp = self.read_eeprom(2);
        mac[4] = (tmp &0xff) as u8;
        mac[5] = (tmp >> 8) as u8;
        EthernetAddress::from_bytes(&mac[..])
    }

    fn write_byte(&self, addr: u16, data: u8) {
        unsafe {
            Port::new(self.io_base + IO_ADDR).write(addr);
            Port::new(self.io_base + IO_DATA).write(data);
        }
    }

    fn write_long(&self, addr: u16, data: u32) {
        unsafe {
            Port::new(self.io_base + IO_ADDR).write(addr);
            Port::new(self.io_base + IO_DATA).write(data);
        }
    }

    fn read_long(&self, addr: u16) -> u32 {
        unsafe {
            Port::new(self.io_base + IO_ADDR).write(addr);
            Port::new(self.io_base + IO_DATA).read()
        }
    }

    fn detect_eeprom(&mut self) {
        self.write_long(EEPROM, 1);
        let mut i = 0;
        while !self.has_eeprom && i < 1000 {
            self.has_eeprom = self.read_long(EEPROM) & 0x10 > 0;
            i += 1;
        }
    }

    fn read_eeprom(&self, addr: u16) -> u32 {
        let e = if self.has_eeprom { 4 } else { 0 };
        self.write_long(EEPROM, 1 | ((addr as u32) << 2 * e));

        let mut res = 0;
        while res & (1 << 1 * e) == 0 {
            res = self.read_long(EEPROM);
            debug!("EEPROM: {:#x} ({:#x})", res, res & (1 << 1 * e));
        }
        res
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
        None
    }

    fn transmit_packet(&mut self, len: usize) {
    }

    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8] {
        &mut self.tx_buffer[0..len]
    }
}
