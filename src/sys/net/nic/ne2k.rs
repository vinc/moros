use crate::sys::net::{Config, EthernetDeviceIO, Stats};
use crate::sys::x86::port::*;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::hint::spin_loop;

const MTU: usize = 1536;

// Page 0 registers
const CR:    u16 = 0x00; // Command Register
const ISR:   u16 = 0x07; // Interrupt Status Register
const RBCR0: u16 = 0x0A; // Remote Byte Count Register 0
const RBCR1: u16 = 0x0B; // Remote Byte Count Register 1
const RCR:   u16 = 0x0C; // Receive Configuration Register
const TCR:   u16 = 0x0C; // Transmit Configuration Register
const DCR:   u16 = 0x0E; // Data Configuration Register
const RESET: u16 = 0x1F;

// Command Register bits
const CR_STP: u8 = 1 << 0; // Stop
const CR_RD2: u8 = 1 << 5; // Abort/Complete Remote DMA

// Interrupt Status Register bits
const ISR_RST: u8 = 1 << 7; // Reset Status

// Receive Configuration Register bits
const RCR_MON: u8 = 1 << 5; // Monitor Mode

// Transmit Configuration Register bits
const TCR_LB0: u8 = 1 << 1; // Loopback Mode

// Data Configuration Register bits
const DCR_WTS: u8 = 1 << 0; // Word Transfer Select
const DCR_LS:  u8 = 1 << 3; // Loopback Select
const DCR_FT1: u8 = 1 << 6; // FIFO threshold select bit 1

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

    fn read(&self, reg: u16) -> u8 {
        unsafe { inb(self.io_base + reg) }
    }

    fn write(&self, reg: u16, value: u8) {
        unsafe { outb(self.io_base + reg, value) }
    }

    fn init(&mut self) {
        // Reset
        self.write(RESET, self.read(RESET));
        while self.read(ISR) & ISR_RST == 0 {
            spin_loop();
        }

        // Program Command Register for page 0
        self.write(CR, CR_STP | CR_RD2); // Stop and Abort DMA

        // Initialize Data Configuration Register
        self.write(DCR, DCR_WTS | DCR_LS | DCR_FT1); // Word-wide DMA transfer

        // Clear Remote Byte Count Registers
        self.write(RBCR0, 0);
        self.write(RBCR1, 0);

        // Initialize Receive Configuration Register
        self.write(RCR, RCR_MON); // Monitor

        // Initialize Transmit Configuration Register
        self.write(TCR, TCR_LB0); // Internal Loopback

        // Clear Interrupt Status Register
        self.write(ISR, 0xFF);

        // Initialize Interrupt Mask Register
        self.write(IMR, 0);
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
