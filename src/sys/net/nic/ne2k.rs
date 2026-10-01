use crate::sys::net::{Config, EthernetDeviceIO, Stats};
use crate::sys::x86::port::*;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp;
use core::hint::spin_loop;
use smoltcp::wire::EthernetAddress;

// Page 0 registers
const CR:     u16 = 0x00; // Command Register
const PSTART: u16 = 0x01; // Page Start Register
const PSTOP:  u16 = 0x02; // Page Stop Register
const BNRY:   u16 = 0x03; // Boundary Register
const TPSR:   u16 = 0x04; // Transmit Page Start Register
const TBCR0:  u16 = 0x05; // Transmit Byte Count Register 0
const TBCR1:  u16 = 0x06; // Transmit Byte Count Register 1
const ISR:    u16 = 0x07; // Interrupt Status Register
const RSAR0:  u16 = 0x08; // Remote Start Address Register 0
const RSAR1:  u16 = 0x09; // Remote Start Address Register 1
const RBCR0:  u16 = 0x0A; // Remote Byte Count Register 0
const RBCR1:  u16 = 0x0B; // Remote Byte Count Register 1
const RCR:    u16 = 0x0C; // Receive Configuration Register
const TCR:    u16 = 0x0D; // Transmit Configuration Register
const DCR:    u16 = 0x0E; // Data Configuration Register
const IMR:    u16 = 0x0F; // Interrupt Mask Register

// Page 1 registers
const PAR0:   u16 = 0x01; // Physical Address Register 0
const CURR:   u16 = 0x07; // Current Page Register
const MAR0:   u16 = 0x08; // Multicast Address Register 0

const DATA:   u16 = 0x10; // Remote DMA Port
const RESET:  u16 = 0x1F; // Reset Port

// Command Register bits
const CR_STP: u8 = 1 << 0; // Stop
const CR_STA: u8 = 1 << 1; // Start
const CR_TXP: u8 = 1 << 2; // Transmit Packet
const CR_RD0: u8 = 1 << 3; // Remote DMA Command bit 0
const CR_RD1: u8 = 1 << 4; // Remote DMA Command bit 1
const CR_RD2: u8 = 1 << 5; // Remote DMA Command bit 2
const CR_PS0: u8 = 1 << 6; // Page Select bit 0

// Interrupt Status Register bits
const ISR_PTX: u8 = 1 << 1; // Packet Transmitted
const ISR_TXE: u8 = 1 << 3; // Transmit Error
const ISR_RDC: u8 = 1 << 6; // Remote DMA Complete
const ISR_RST: u8 = 1 << 7; // Reset Status

// Receive Configuration Register bits
const RCR_AB:  u8 = 1 << 2; // Accept Broadcast
const RCR_MON: u8 = 1 << 5; // Monitor Mode

// Transmit Configuration Register bits
const TCR_LB0: u8 = 1 << 1; // Loopback Mode

// Data Configuration Register bits
const DCR_WTS: u8 = 1 << 0; // Word Transfer Select
const DCR_LS:  u8 = 1 << 3; // Loopback Select
const DCR_FT1: u8 = 1 << 6; // FIFO threshold select bit 1

const TX_START: u8 = 0x40; // Transmit buffer start page
const RX_START: u8 = 0x4C; // Receive buffer ring start page
const RX_STOP:  u8 = 0x80; // Receive buffer ring stop page (exclusive)

const MIN_PACKET: usize = 60;

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
            tx_buffer: Vec::new(),
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

    fn read_buffer(&self, addr: u16, len: usize) -> Vec<u8> {
        let n = len.next_multiple_of(2);

        let rbcr = n.to_le_bytes();
        self.write(RBCR0, rbcr[0]);
        self.write(RBCR1, rbcr[1]);

        let rsar = addr.to_le_bytes();
        self.write(RSAR0, rsar[0]);
        self.write(RSAR1, rsar[1]);

        self.write(CR, CR_STA | CR_RD0); // Remote Read

        let mut buf = Vec::with_capacity(n);
        for _ in 0..(n / 2) {
            let data = unsafe { inw(self.io_base + DATA) };
            buf.extend_from_slice(&data.to_le_bytes());
        }
        while self.read(ISR) & ISR_RDC == 0 {
            spin_loop()
        }
        self.write(ISR, ISR_RDC);
        buf.truncate(len);
        buf
    }

    fn write_buffer(&self, addr: u16, buf: &[u8]) {
        let n = buf.len().next_multiple_of(2);

        let rbcr = n.to_le_bytes();
        self.write(RBCR0, rbcr[0]);
        self.write(RBCR1, rbcr[1]);

        let rsar = addr.to_le_bytes();
        self.write(RSAR0, rsar[0]);
        self.write(RSAR1, rsar[1]);

        self.write(CR, CR_STA | CR_RD1); // Remote Write

        for chunk in buf.chunks(2) {
            let data = match *chunk {
                [b0, b1] => u16::from_le_bytes([b0, b1]),
                [b0] => u16::from(b0), // Last chunk of an odd-length buffer
                _ => unreachable!(),
            };
            unsafe { outw(self.io_base + DATA, data) };
        }
        while self.read(ISR) & ISR_RDC == 0 {
            spin_loop()
        }
        self.write(ISR, ISR_RDC);
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

        // Initialize Receive Buffer Ring
        self.write(BNRY, RX_START);
        self.write(PSTART, RX_START);
        self.write(PSTOP, RX_STOP);

        // Clear Interrupt Status Register
        self.write(ISR, 0xFF);

        // Initialize Interrupt Mask Register
        self.write(IMR, 0);

        // Read MAC address from PROM (discard high byte of each word)
        let prom = self.read_buffer(0, 12);
        let mac: [u8; 6] = core::array::from_fn(|i| prom[i * 2]);
        self.config.update_mac(EthernetAddress::from_bytes(&mac));

        // Program Command Register for page 1
        self.write(CR, CR_PS0 | CR_STP | CR_RD2);

        // Initialize Physical Address Registers
        for i in 0..6 {
            self.write(PAR0 + i, mac[i as usize]);
        }

        // Initialize Multicast Address Registers
        for i in 0..8 {
            self.write(MAR0 + i, 0xFF);
        }

        // Initialize Current Pointer
        self.write(CURR, RX_START + 1);

        // Program Command Register for page 0
        self.write(CR, CR_STA | CR_RD2); // Start and Abort DMA

        self.write(TCR, 0); // Normal Operation
        self.write(RCR, RCR_AB); // Accept Broadcast
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
        // Read Current Page Register
        self.write(CR, CR_PS0 | CR_STP | CR_RD2); // Page 1
        let curr = self.read(CURR);
        self.write(CR, CR_STA | CR_RD2); // Page 0

        // Find the page of the next packet
        let mut page = self.read(BNRY) + 1;
        if page == RX_STOP {
            page = RX_START;
        }
        if page == curr {
            return None; // Receive buffer ring is empty
        }

        // Read packet header
        let addr = (page as u16) << 8;
        let header = self.read_buffer(addr, 4);
        let next = header[1];
        let len = u16::from_le_bytes([header[2], header[3]]) as usize;

        // Read packet
        let packet = self.read_buffer(addr + 4, len - 4);

        // Update Boundary Register
        let bnry = if next == RX_START { RX_STOP } else { next };
        self.write(BNRY, bnry - 1);

        Some(packet)
    }

    fn transmit_packet(&mut self, len: usize) {
        let len = cmp::max(MIN_PACKET, len);

        // Write packet
        self.write_buffer((TX_START as u16) << 8, &self.tx_buffer[..len]);

        // Set Transmit Page Start Register
        self.write(TPSR, TX_START);

        // Set Transmit Byte Count
        let tbcr = len.to_le_bytes();
        self.write(TBCR0, tbcr[0]);
        self.write(TBCR1, tbcr[1]);

        // Start transmission
        self.write(CR, CR_STA | CR_TXP | CR_RD2);

        while self.read(ISR) & (ISR_PTX | ISR_TXE) == 0 {
            spin_loop()
        }
        self.write(ISR, ISR_PTX | ISR_TXE);
    }

    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8] {
        self.tx_buffer.clear();
        self.tx_buffer.resize(cmp::max(MIN_PACKET, len), 0);
        &mut self.tx_buffer[0..len]
    }
}
