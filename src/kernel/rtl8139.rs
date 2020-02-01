use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::vec;
use core::cell::RefCell;
use core::convert::TryInto;
use crate::{print, kernel};
use crate::user;
use lazy_static::lazy_static;
use smoltcp::Result;
use smoltcp::iface::{EthernetInterface, EthernetInterfaceBuilder, NeighborCache, Routes};
use smoltcp::phy::{Device, DeviceCapabilities};
use smoltcp::phy;
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpCidr, Ipv4Address};
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::VirtAddr;

// 00 = 8K + 16 bytes
// 01 = 16K + 16 bytes
// 10 = 32K + 16 bytes
// 11 = 64K + 16 bytes
const RX_BUFFER_IDX: usize = 0;

const MTU: usize = 1500;

const RX_BUFFER_PAD: usize = 16;
const RX_BUFFER_LEN: usize = (8129 << RX_BUFFER_IDX) + RX_BUFFER_PAD;

const TX_BUFFER_LEN: usize = 4096;
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
const TCR_MXDMA0: u32 = 1 << 8;
const TCR_MXDMA2: u32 = 1 << 9;
const TCR_MXDMA1: u32 = 1 << 10;

// Interrupt Mask Register
const IMR_TOK: u16 = 1 << 2; // Transmit OK Interrupt
const IMR_ROK: u16 = 1 << 0; // Receive OK Interrupt

lazy_static! {
    pub static ref IFACE: Mutex<Option<EthernetInterface<'static, 'static, 'static, RTL8139>>> = Mutex::new(None);
}

pub struct Ports {
    pub idr: [Port<u8>; 6], // ID Registers (IDR0 ... IDR5)
    pub tx_cmds: [Port<u32>; TX_BUFFERS_COUNT], // Transmit Status of Descriptors (TSD0 .. TSD3)
    pub tx_addrs: [Port<u32>; TX_BUFFERS_COUNT], // Transmit Start Address of Descriptor0 (TSAD0 .. TSAD3)
    pub config1: Port<u8>, // Configuration Register 1 (CONFIG1)
    pub rx_addr: Port<u32>, // Receive (Rx) Buffer Start Address (RBSTART)
    pub rx_ptr: Port<u16>, // Current Address of Packet Read (CAPR)
    pub cmd: Port<u8>, // Command Register (CR)
    pub imr: Port<u16>, // Interrupt Mask Register (IMR)
    pub isr: Port<u16>, // Interrupt Status Register (ISR)
    pub tx_config: Port<u32>, // Transmit (Tx) Configuration Register (TCR)
    pub rx_config: Port<u32>, // Receive (Rx) Configuration Register (RCR)
}

impl Ports {
    pub fn new(io_addr: u16) -> Self {
        Self {
            idr: [
                Port::new(io_addr + 0x00),
                Port::new(io_addr + 0x01),
                Port::new(io_addr + 0x02),
                Port::new(io_addr + 0x03),
                Port::new(io_addr + 0x04),
                Port::new(io_addr + 0x05),
            ],
            tx_cmds: [
                Port::new(io_addr + 0x10),
                Port::new(io_addr + 0x14),
                Port::new(io_addr + 0x18),
                Port::new(io_addr + 0x1C),
            ],
            tx_addrs: [
                Port::new(io_addr + 0x20),
                Port::new(io_addr + 0x24),
                Port::new(io_addr + 0x28),
                Port::new(io_addr + 0x2C),
            ],
            config1: Port::new(io_addr + 0x52),
            rx_addr: Port::new(io_addr + 0x30),
            rx_ptr: Port::new(io_addr + 0x38), // CAPR
            cmd: Port::new(io_addr + 0x37),
            imr: Port::new(io_addr + 0x3C),
            isr: Port::new(io_addr + 0x3E),
            tx_config: Port::new(io_addr + 0x40),
            rx_config: Port::new(io_addr + 0x44),
        }
    }
}

pub struct State {
    rx_bytes_count: u64,
    tx_bytes_count: u64,
    rx_packets_count: u64,
    tx_packets_count: u64,
}

pub struct RTL8139 {
    state: RefCell<State>,
    ports: Ports,
    eth_addr: Option<EthernetAddress>,
    rx_buffer: Box<Vec<u8>>,
    rx_offset: usize,
    tx_buffers: [Box<Vec<u8>>; TX_BUFFERS_COUNT], // TODO: Remove this
    tx_id: usize,
    pub debug_mode: bool,
}

impl RTL8139 {
    pub fn new(io_addr: u16) -> Self {
        let state = State {
            rx_bytes_count: 0,
            tx_bytes_count: 0,
            rx_packets_count: 0,
            tx_packets_count: 0,
        };
        Self {
            state: RefCell::new(state),
            ports: Ports::new(io_addr),
            eth_addr: None,

            // Add MTU to RX_BUFFER_LEN if RCR_WRAP is set
            rx_buffer: Box::new(vec![0; RX_BUFFER_LEN + MTU]),

            rx_offset: 0,
            tx_buffers: [
                Box::new(vec![0; TX_BUFFER_LEN]),
                Box::new(vec![0; TX_BUFFER_LEN]),
                Box::new(vec![0; TX_BUFFER_LEN]),
                Box::new(vec![0; TX_BUFFER_LEN]),
            ],

            // Before a transmission begin the id is incremented,
            // so the first transimission will start at 0.
            tx_id: TX_BUFFERS_COUNT - 1,

            debug_mode: false,
        }
    }

    pub fn init(&mut self) {
        // Power on
        unsafe { self.ports.config1.write(0) }

        // Software reset
        unsafe {
            self.ports.cmd.write(CR_RST);
            while self.ports.cmd.read() & CR_RST != 0 {}
        }

        // Enable Receive and Transmitter
        unsafe { self.ports.cmd.write(CR_RE | CR_TE) }

        // Read MAC addr
        let mac = unsafe {
            [
                self.ports.idr[0].read(),
                self.ports.idr[1].read(),
                self.ports.idr[2].read(),
                self.ports.idr[3].read(),
                self.ports.idr[4].read(),
                self.ports.idr[5].read(),
            ]
        };
        self.eth_addr = Some(EthernetAddress::from_bytes(&mac));

        // Get physical address of rx_buffer
        let rx_ptr = &self.rx_buffer[0] as *const u8;
        let virt_addr = VirtAddr::new(rx_ptr as u64);
        let phys_addr = kernel::mem::translate_addr(virt_addr).unwrap();
        let rx_addr = phys_addr.as_u64();

        // Init Receive buffer
        unsafe { self.ports.rx_addr.write(rx_addr as u32) }

        for i in 0..4 {
            // Get physical address of each tx_buffer
            let tx_ptr = &self.tx_buffers[i][0] as *const u8;
            let virt_addr = VirtAddr::new(tx_ptr as u64);
            let phys_addr = kernel::mem::translate_addr(virt_addr).unwrap();
            let tx_addr = phys_addr.as_u64();

            // Init Transmit buffer
            unsafe { self.ports.tx_addrs[i].write(tx_addr as u32) }
        }

        // Set interrupts
        unsafe { self.ports.imr.write(IMR_TOK | IMR_ROK) }

        // Configure receive buffer (RCR)
        unsafe { self.ports.rx_config.write(RCR_RBLEN | RCR_WRAP | RCR_AB | RCR_AM | RCR_APM | RCR_AAP) }

        // Configure transmit buffer (TCR)
        unsafe { self.ports.tx_config.write(TCR_IFG | TCR_MXDMA0 | TCR_MXDMA1 | TCR_MXDMA2); }
    }

    pub fn rx_bytes_count(&self) -> u64 {
        self.state.borrow().rx_bytes_count
    }
    pub fn tx_bytes_count(&self) -> u64 {
        self.state.borrow().tx_bytes_count
    }
    pub fn rx_packets_count(&self) -> u64 {
        self.state.borrow().rx_packets_count
    }
    pub fn tx_packets_count(&self) -> u64 {
        self.state.borrow().tx_packets_count
    }
}

impl<'a> Device<'a> for RTL8139 {
    type RxToken = RxToken<'a>;
    type TxToken = TxToken<'a>;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = MTU;
        caps.max_burst_size = Some(1);
        caps
    }

    // RxToken buffer, when not empty, will contains:
    // [header            (2 bytes)]
    // [length            (2 bytes)]
    // [packet   (length - 4 bytes)]
    // [crc               (4 bytes)]
    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let cmd = unsafe { self.ports.cmd.read() };
        if (cmd & CR_BUFE) == CR_BUFE {
            return None;
        }
        //let isr = unsafe { self.ports.isr.read() };
        let offset = self.rx_offset;
        let header = u16::from_le_bytes(self.rx_buffer[(offset + 0)..(offset + 2)].try_into().unwrap());
        if self.debug_mode {
            print!("------------------------------------------------------------------\n");
            let uptime = kernel::clock::clock_monotonic();
            print!("[{:.6}] NET RTL8139 receiving buffer:\n\n", uptime);
            print!("Command Register: 0x{:02X}\n", cmd);
            //print!("Interrupt Status Register: 0x{:02X}\n", isr);
            print!("Header: 0x{:04X}\n", header);
        }
        if header & ROK != ROK {
            return None;
        }
        let length = u16::from_le_bytes(self.rx_buffer[(offset + 2)..(offset + 4)].try_into().unwrap());
        let n = length as usize;
        let crc = u32::from_le_bytes(self.rx_buffer[(offset + n)..(offset + n + 4)].try_into().unwrap());

        if self.debug_mode {
            print!("Length: {} bytes\n", n - 4);
            print!("CRC: 0x{:08X}\n", crc);
            print!("RX Offset: {}\n", self.rx_offset);
            user::hex::print_hex(&self.rx_buffer[(offset + 4)..(offset + n)]);
        }

        // Update buffer read pointer
        self.rx_offset = (offset + n + 4 + 3) & !3;
        unsafe { self.ports.rx_ptr.write((self.rx_offset - RX_BUFFER_PAD) as u16); }
        self.rx_offset %= RX_BUFFER_LEN;

        let state = &self.state;
        let rx = RxToken { state, buffer: &mut self.rx_buffer[(offset + 4)..(offset + n)] };
        let cmd_port = self.ports.tx_cmds[self.tx_id].clone();
        let buffer = &mut self.tx_buffers[self.tx_id];
        let debug_mode = self.debug_mode;
        let tx = TxToken { state, cmd_port, buffer, debug_mode };

        Some((rx, tx))
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        self.tx_id = (self.tx_id + 1) % TX_BUFFERS_COUNT;

        if self.debug_mode {
            print!("------------------------------------------------------------------\n");
            let uptime = kernel::clock::clock_monotonic();
            print!("[{:.6}] NET RTL8139 transmitting buffer:\n\n", uptime);
            print!("TX Buffer: {}\n", self.tx_id);
        }

        let state = &self.state;
        let cmd_port = self.ports.tx_cmds[self.tx_id].clone();
        let buffer = &mut self.tx_buffers[self.tx_id];
        let debug_mode = self.debug_mode;
        let tx = TxToken { state, cmd_port, buffer, debug_mode };

        Some(tx)
    }
}

#[doc(hidden)]
pub struct RxToken<'a> {
    state: &'a RefCell<State>,
    buffer: &'a mut [u8]
}

impl<'a> phy::RxToken for RxToken<'a> {
    fn consume<R, F>(self, _timestamp: Instant, f: F) -> Result<R>
        where F: FnOnce(&mut [u8]) -> Result<R>
    {
        self.state.borrow_mut().rx_packets_count += 1;
        self.state.borrow_mut().rx_bytes_count += self.buffer.len() as u64;
        f(self.buffer)
    }
}

#[doc(hidden)]
pub struct TxToken<'a> {
    state: &'a RefCell<State>,
    cmd_port: Port<u32>,
    buffer: &'a mut [u8],
    debug_mode: bool,
}

//const CRS: u32 = 1 << 31; // Carrier Sense Lost
//const TAB: u32 = 1 << 30; // Transmit Abort
//const OWC: u32 = 1 << 29; // Out of Window Collision
//const CDH: u32 = 1 << 28; // CD Heart Beat
const TOK: u32 = 1 << 15; // Transmit OK
//const TUN: u32 = 1 << 14; // Transmit FIFO Underrun
const OWN: u32 = 1 << 13; // DMA operation completed

impl<'a> phy::TxToken for TxToken<'a> {
    fn consume<R, F>(mut self, _timestamp: Instant, len: usize, f: F) -> Result<R>
        where F: FnOnce(&mut [u8]) -> Result<R>
    {
        // 1. Copy the packet to a physically continuous buffer in memory.
        let res = f(&mut self.buffer[0..len]);

        // 2. Fill in Start Address(physical address) of this buffer.
        // NOTE: This has was done during init

        if res.is_ok() {
            unsafe {
                // 3. Fill in Transmit Status: the size of this packet, the
                // early transmit threshold, and clear OWN bit in TSD (this
                // starts the PCI operation).
                // NOTE: The length of the packet use the first 13 bits (but
                // should not exceed 1792 bytes), and a value of 0x000000
                // for the early transmit threshold means 8 bytes. So we
                // just write the size of the packet.
                self.cmd_port.write(0x1FFF & len as u32);

                // 4. When the whole packet is moved to FIFO, the OWN bit is
                // set to 1.
                while self.cmd_port.read() & OWN != OWN {}
                // 5. When the whole packet is moved to line, the TOK bit is
                // set to 1.
                while self.cmd_port.read() & TOK != TOK {}
            }
        }
        self.state.borrow_mut().tx_packets_count += 1;
        self.state.borrow_mut().tx_bytes_count += len as u64;
        if self.debug_mode {
            user::hex::print_hex(&self.buffer[0..len]);
        }

        res
    }
}

pub fn init() {
    if let Some(mut pci_device) = kernel::pci::find_device(0x10EC, 0x8139) {
        pci_device.enable_bus_mastering();

        let irq = pci_device.interrupt_line;
        let io_addr = (pci_device.base_addresses[0] as u16) & 0xFFF0;
        let mut rtl8139_device = RTL8139::new(io_addr);

        rtl8139_device.init();

        if let Some(eth_addr) = rtl8139_device.eth_addr {
            let uptime = kernel::clock::clock_monotonic();
            print!("[{:.6}] NET RTL8139 MAC {}\n", uptime, eth_addr);

            let neighbor_cache = NeighborCache::new(BTreeMap::new());
            let routes = Routes::new(BTreeMap::new());
            let ip_addrs = [IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0)];
            let iface = EthernetInterfaceBuilder::new(rtl8139_device).
                ethernet_addr(eth_addr).
                neighbor_cache(neighbor_cache).
                ip_addrs(ip_addrs).
                routes(routes).
                finalize();

            *IFACE.lock() = Some(iface);
        }

        kernel::idt::set_irq_handler(irq, interrupt_handler);
    }
}

pub fn interrupt_handler() {
    print!("RTL8139 interrupt!");
    if let Some(ref mut iface) = *IFACE.lock() {
        unsafe { iface.device_mut().ports.isr.write(0x1) } // Clear the interrupt
    }
}
