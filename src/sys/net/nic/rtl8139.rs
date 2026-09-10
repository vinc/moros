use crate::sys::mem::PhysBuf;
use crate::sys::net::{Config, EthernetDeviceIO, Stats};
use crate::sys::x86::port::*;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::convert::TryInto;
use core::hint::spin_loop;
use core::sync::atomic::{fence, AtomicUsize, Ordering};
use smoltcp::wire::EthernetAddress;

// 00 = 8K + 16 bytes
// 01 = 16K + 16 bytes
// 10 = 32K + 16 bytes
// 11 = 64K + 16 bytes
const RX_BUFFER_IDX: usize = 0;

const MTU: usize = 1536;

const RX_BUFFER_PAD: usize = 16;
const RX_BUFFER_LEN: usize = 8192 << RX_BUFFER_IDX;

const TX_BUFFER_LEN: usize = 2048;
const TX_BUFFERS_COUNT: usize = 4;
const ROK: u16 = 0x01;

const CR_RST: u8 = 1 << 4; // Reset
const CR_RE: u8 = 1 << 3; // Receiver Enable
const CR_TE: u8 = 1 << 2; // Transmitter Enable
const CR_BUFE: u8 = 1 << 0; // Buffer Empty

// Rx Buffer Length
const RCR_RBLEN: u32 = (RX_BUFFER_IDX << 11) as u32;

// When the WRAP bit is set, the nic will keep moving the rest
// of the packet data into the memory immediately after the
// end of the Rx buffer instead of going back to the begining
// of the buffer. So the buffer must have an additionnal 1500 bytes.
const RCR_WRAP: u32 = 1 << 7;

const RCR_AB: u32 = 1 << 3; // Accept Broadcast packets
const RCR_AM: u32 = 1 << 2; // Accept Multicast packets
const RCR_APM: u32 = 1 << 1; // Accept Physical Match packets
const RCR_AAP: u32 = 1 << 0; // Accept All Packets

// Interframe Gap Time
const TCR_IFG: u32 = 3 << 24;

// Max DMA Burst Size per Tx DMA Burst
// 000 = 16 bytes
// 001 = 32 bytes
// 010 = 64 bytes
// 011 = 128 bytes
// 100 = 256 bytes
// 101 = 512 bytes
// 110 = 1024 bytes
// 111 = 2048 bytes
//const TCR_MXDMA0: u32 = 1 << 8;
const TCR_MXDMA1: u32 = 1 << 9;
const TCR_MXDMA2: u32 = 1 << 10;

// Interrupt Mask Register
//const IMR_TOK: u16 = 1 << 2; // Transmit OK Interrupt
//const IMR_ROK: u16 = 1 << 0; // Receive OK Interrupt

//const CRS: u32 = 1 << 31; // Carrier Sense Lost
//const TAB: u32 = 1 << 30; // Transmit Abort
//const OWC: u32 = 1 << 29; // Out of Window Collision
//const CDH: u32 = 1 << 28; // CD Heart Beat
const TOK: u32 = 1 << 15; // Transmit OK
                          //const TUN: u32 = 1 << 14; // Transmit FIFO Underrun
const OWN: u32 = 1 << 13; // DMA operation completed

// Registers
const TSD0:    u16 = 0x10; // Transmit Status of Descriptors 0
const TSAD0:   u16 = 0x20; // Transmit Start Address of Descriptor 0
const RBSTART: u16 = 0x30; // Receive (Rx) Buffer Start Address
const CR:      u16 = 0x37; // Command Register
const CAPR:    u16 = 0x38; // Current Address of Packet Read
const CBA:     u16 = 0x3A; // Current Buffer Address
//const IMR:     u16 = 0x3C; // Interrupt Mask Register
//const ISR:     u16 = 0x3E; // Interrupt Status Register
const TCR:     u16 = 0x40; // Transmit (Tx) Configuration Register
const RCR:     u16 = 0x44; // Receive (Rx) Configuration Register
const CONFIG0: u16 = 0x51; // Configuration 0

#[derive(Clone)]
pub struct Ports {
    io_base: u16,
}

impl Ports {
    pub fn new(io_base: u16) -> Self {
        Self { io_base }
    }

    fn mac(&self) -> [u8; 6] {
        // Read the EEPROM EthernetID loaded in the ID Registers
        core::array::from_fn(|i| unsafe {
            inb(self.io_base + i as u16)
        })
    }

    pub fn read_tsd(&self, i: usize) -> u32 {
        debug_assert!(i < 4);
        unsafe { inl(self.io_base + TSD0 + 4 * i as u16) }
    }

    pub fn write_tsd(&self, i: usize, value: u32) {
        debug_assert!(i < 4);
        unsafe { outl(self.io_base + TSD0 + 4 * i as u16, value) }
    }

    pub fn write_tsad(&self, i: usize, value: u32) {
        debug_assert!(i < 4);
        unsafe { outl(self.io_base + TSAD0 + 4 * i as u16, value) }
    }

    pub fn write_rbstart(&self, value: u32) {
        unsafe { outl(self.io_base + RBSTART, value) }
    }

    pub fn read_cr(&self) -> u8 {
        unsafe { inb(self.io_base + CR) }
    }

    pub fn write_cr(&self, value: u8) {
        unsafe { outb(self.io_base + CR, value) }
    }

    pub fn read_capr(&self) -> u16 {
        unsafe { inw(self.io_base + CAPR) }
    }

    pub fn write_capr(&self, value: u16) {
        unsafe { outw(self.io_base + CAPR, value) }
    }

    pub fn read_cba(&self) -> u16 {
        unsafe { inw(self.io_base + CBA) }
    }

    /*
    pub fn write_imr(&self, value: u16) {
        unsafe { outw(self.io_base + IMR, value) }
    }

    pub fn write_isr(&self, value: u16) {
        unsafe { outw(self.io_base + ISR, value) }
    }
    */

    pub fn write_tcr(&self, value: u32) {
        unsafe { outl(self.io_base + TCR, value) }
    }

    pub fn write_rcr(&self, value: u32) {
        unsafe { outl(self.io_base + RCR, value) }
    }

    pub fn write_config(&self, i: usize, value: u8) {
        debug_assert!(i < 2);
        unsafe { outb(self.io_base + CONFIG0 + i as u16, value) }
    }
}

#[derive(Clone)]
pub struct Device {
    config: Arc<Config>,
    stats: Arc<Stats>,
    ports: Ports,

    rx_buffer: PhysBuf,
    rx_offset: usize,
    tx_buffers: [PhysBuf; TX_BUFFERS_COUNT],
    tx_id: Arc<AtomicUsize>,
}

impl Device {
    pub fn new(io_base: u16) -> Self {
        let mut device = Self {
            config: Arc::new(Config::new()),
            stats: Arc::new(Stats::new()),
            ports: Ports::new(io_base),

            // Add MTU to RX_BUFFER_LEN if RCR_WRAP is set
            rx_buffer: PhysBuf::new(RX_BUFFER_LEN + RX_BUFFER_PAD + MTU),

            rx_offset: 0,
            tx_buffers: [(); TX_BUFFERS_COUNT].map(|_|
                PhysBuf::new(TX_BUFFER_LEN)
            ),

            // Before a transmission begin the id is incremented,
            // so the first transimission will start at 0.
            tx_id: Arc::new(AtomicUsize::new(TX_BUFFERS_COUNT - 1)),
        };
        device.init();
        device
    }

    /*
    fn clear_interrupts(&mut self) {
        let isr = unsafe { self.ports.isr.read() };
        if isr != 0 {
            unsafe { self.ports.isr.write(isr) }
        }
    }
    */

    fn init(&mut self) {
        // Power on
        self.ports.write_config(1, 0);

        // Software reset
        self.ports.write_cr(CR_RST);
        fence(Ordering::SeqCst);
        while self.ports.read_cr() & CR_RST != 0 {
            spin_loop();
        }

        //self.clear_interrupts();

        // Enable interrupts
        // self.ports.write_imr(IMR_TOK | IMR_ROK);

        // Enable receiver and transmitter
        self.ports.write_cr(CR_RE | CR_TE);

        // Read MAC addr
        self.config.update_mac(EthernetAddress::from_bytes(&self.ports.mac()));

        // Get physical address of rx_buffer
        let rx_addr = self.rx_buffer.addr();

        // Init Receive buffer
        self.ports.write_rbstart(rx_addr as u32);

        for i in 0..4 {
            // Get physical address of each tx_buffer
            let tx_addr = self.tx_buffers[i].addr();

            // Init Transmit buffer
            self.ports.write_tsad(i, tx_addr as u32);
        }

        // Configure receive buffer (RCR)
        let flags = RCR_RBLEN | RCR_WRAP | RCR_AB | RCR_AM | RCR_APM | RCR_AAP;
        self.ports.write_rcr(flags);

        // Configure transmit buffer (TCR)
        let flags = TCR_IFG | TCR_MXDMA1 | TCR_MXDMA2;
        self.ports.write_tcr(flags);
    }
}

impl EthernetDeviceIO for Device {
    fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    fn stats(&self) -> Arc<Stats> {
        self.stats.clone()
    }

    // RxToken buffer, when not empty, will contains:
    // [header            (2 bytes)]
    // [length            (2 bytes)]
    // [packet   (length - 4 bytes)]
    // [crc               (4 bytes)]
    fn receive_packet(&mut self) -> Option<Vec<u8>> {
        //self.clear_interrupts();

        let cmd = self.ports.read_cr();
        if (cmd & CR_BUFE) == CR_BUFE {
            return None;
        }

        let cba = self.ports.read_cba();

        // CAPR starts at 65520 and with the pad it overflows to 0
        let capr = self.ports.read_capr();
        let offset = ((capr as usize) + RX_BUFFER_PAD) % (1 << 16);

        let header = u16::from_le_bytes(
            self.rx_buffer[(offset + 0)..(offset + 2)].try_into().unwrap(),
        );

        if header & ROK != ROK {
            let capr = ((cba as usize) % RX_BUFFER_LEN) - RX_BUFFER_PAD;
            self.ports.write_capr(capr as u16);
            return None;
        }

        let n = u16::from_le_bytes(
            self.rx_buffer[(offset + 2)..(offset + 4)].try_into().unwrap()
        ) as usize;

        // Update buffer read pointer
        self.rx_offset = (offset + n + 4 + 3) & !3;
        let capr = (self.rx_offset % RX_BUFFER_LEN) - RX_BUFFER_PAD;
        self.ports.write_capr(capr as u16);

        Some(self.rx_buffer[(offset + 4)..(offset + n)].to_vec())
    }

    fn transmit_packet(&mut self, len: usize) {
        let tx_id = self.tx_id.load(Ordering::SeqCst);

        // RTL8139 will not transmit packets smaller than 64 bits
        let len = len.max(60); // 60 + 4 bits of CRC

        // Fill in Transmit Status: the size of this packet, the early
        // transmit threshold, and clear OWN bit in TSD (this starts the
        // PCI operation).
        // NOTE: The length of the packet use the first 13 bits (but should
        // not exceed 1792 bytes), and a value of 0x000000 for the early
        // transmit threshold means 8 bytes. So we just write the size of
        // the packet.
        self.ports.write_tsd(tx_id, 0x1FFF & len as u32);
        fence(Ordering::SeqCst);

        while self.ports.read_tsd(tx_id) & OWN != OWN {
            spin_loop();
        }
        while self.ports.read_tsd(tx_id) & TOK != TOK {
            spin_loop();
        }
        //self.ports.write_isr(0x4);
    }

    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8] {
        let tx_id = (self.tx_id.load(Ordering::SeqCst) + 1) % TX_BUFFERS_COUNT;
        self.tx_id.store(tx_id, Ordering::SeqCst);
        &mut self.tx_buffers[tx_id][0..len]
    }
}

/*
pub fn interrupt_handler() {
    printk!("RTL8139 interrupt!\n");
    if let Some(mut guard) = sys::net::IFACE.try_lock() {
        if let Some(ref mut iface) = *guard {
            // Clear the interrupt
            iface.device_mut().ports.write_isr(0xFFFF);
        }
    }
}
*/
