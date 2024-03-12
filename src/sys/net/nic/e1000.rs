use crate::sys;
use crate::sys::allocator::PhysBuf;
use crate::sys::net::{EthernetDeviceIO, Config, Stats};

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use smoltcp::wire::EthernetAddress;
use x86_64::instructions::port::Port;
use x86_64::PhysAddr;

const REG_EEPROM: u16 = 0x0014;

const IO_ADDR: u16 = 0x00;
const IO_DATA: u16 = 0x04;

const MTU: usize = 1500;
const RX_BUFFERS_COUNT: usize = 32;
const TX_BUFFERS_COUNT: usize = 8;

#[derive(Clone)]
pub struct Device {
    mem_base: PhysAddr,
    io_base: u16,
    bar_type: u16,
    has_eeprom: bool,
    config: Arc<Config>,
    stats: Arc<Stats>,
    rx_buffers: [PhysBuf; RX_BUFFERS_COUNT],
    tx_buffers: [PhysBuf; TX_BUFFERS_COUNT],
    rx_id: Arc<AtomicUsize>,
    tx_id: Arc<AtomicUsize>,
}

impl Device {
    pub fn new(io_base: u16, mem_base: PhysAddr, bar_type: u16) -> Self {
        let mut device = Self {
            bar_type: bar_type,
            io_base: io_base,
            mem_base: mem_base,
            has_eeprom: false,
            config: Arc::new(Config::new()),
            stats: Arc::new(Stats::new()),
            rx_buffers: [(); RX_BUFFERS_COUNT].map(|_| PhysBuf::new(MTU)),
            tx_buffers: [(); TX_BUFFERS_COUNT].map(|_| PhysBuf::new(MTU)),
            rx_id: Arc::new(AtomicUsize::new(0)),
            tx_id: Arc::new(AtomicUsize::new(0)),
        };
        device.init();
        device
    }

    fn init(&mut self) {
        //self.write_long(CTRL, CTRL_RST);
        //self.config.update_mac(EthernetAddress::from_bytes(&[0, 0, 0, 0, 0, 0]));
        self.detect_eeprom();
        self.config.update_mac(self.read_mac());

        // TODO:
        // - Receive Initialization
        //   - Set MAC address in Receive Address (RAL/RAH) register
        //   - Set 0b in Multicast Table Array (MTA) register
        //   - Set Interrupt Mask Set/Read (IMS) register
        //   - Allocate a region of memory for the receive descriptor list
        //   - Set Receive Descriptor Base Address (RDBAL/RDBAH) registers
        //   - Set Receive Descriptor Length (RDLEN) register
        //   - Set Receive Descriptor Head (RDH) register
        //   - Set Receive Descriptor Tail (RDT) register
        //   - Set Receive Control (RCTL) register
        // - Transmit Initialization
        //   - Allocate a region of memory for the transmit descriptor list
        //   - Set Transmit Descriptor Base Address (TDBAL/TDBAH) registers
        //   - Set Transmit Descriptor Length (TDLEN) register
        //   - Set Transmit Descriptor Head (TDH) register
        //   - Set Transmit Descriptor Tail (TDT) register
        //   - Set Transmit Control (RCTL) register
    }

    fn read_mac(&self) -> EthernetAddress {
        let mut mac = [0; 6];
        if self.has_eeprom {
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
        } else {
            unsafe {
                let addr = sys::mem::phys_to_virt(self.mem_base + 0x5400 as u64).as_u64();
                let mac_32 = core::ptr::read_volatile(addr as *const u32);
                if mac_32 != 0 {
                    let mac_8 = alloc::slice::from_raw_parts(addr as *const u8, 6);
                    mac[..].clone_from_slice(mac_8);
                }
            }
        }
        EthernetAddress::from_bytes(&mac[..])
    }

    fn write(&self, addr: u16, data: u32) {
        unsafe {
            if self.bar_type == 0 {
                let addr = sys::mem::phys_to_virt(self.mem_base + addr as u64).as_u64() as *mut u32;
                core::ptr::write_volatile(addr, data);
            } else {
                Port::new(self.io_base + IO_ADDR).write(addr);
                Port::new(self.io_base + IO_DATA).write(data);
            }
        }
    }

    fn read(&self, addr: u16) -> u32 {
        unsafe {
            if self.bar_type == 0 {
                let addr = sys::mem::phys_to_virt(self.mem_base + addr as u64).as_u64() as *const u32;
                core::ptr::read_volatile(addr)
            } else {
                Port::new(self.io_base + IO_ADDR).write(addr);
                Port::new(self.io_base + IO_DATA).read()
            }
        }
    }

    fn detect_eeprom(&mut self) {
        self.write(REG_EEPROM, 1);
        let mut i = 0;
        while !self.has_eeprom && i < 1000 {
            self.has_eeprom = self.read(REG_EEPROM) & 0x10 > 0;
            i += 1;
        }
    }

    fn read_eeprom(&self, addr: u16) -> u32 {
        let e = if self.has_eeprom { 4 } else { 0 };
        self.write(REG_EEPROM, 1 | ((addr as u32) << 2 * e));

        let mut res = 0;
        while res & (1 << 1 * e) == 0 {
            res = self.read(REG_EEPROM);
        }
        (res >> 16) & 0xFFFF
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
        let tx_id = self.tx_id.load(Ordering::SeqCst);
        &mut self.tx_buffers[tx_id][0..len]
    }
}
