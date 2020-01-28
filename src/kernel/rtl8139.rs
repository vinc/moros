use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::vec;
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

const RX_BUFFER_LEN: usize = 8192 + 16;
const RX_BUFFER_EMPTY: u8 = 0x01;
const TX_BUFFER_LEN: usize = 4096 + 16;
const TX_BUFFERS_COUNT: usize = 4;
const ISR_ROK: u16 = 0x01;

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

lazy_static! {
    pub static ref IFACE: Mutex<Option<EthernetInterface<'static, 'static, 'static, RTL8139>>> = Mutex::new(None);
}

pub struct Ports {
    pub idr: [Port<u8>; 6],
    pub tx_cmds: [Port<u32>; TX_BUFFERS_COUNT],
    pub tx_addrs: [Port<u32>; TX_BUFFERS_COUNT],
    pub config1: Port<u8>,
    pub rx_addr: Port<u32>,
    pub rx_ptr: Port<u16>,
    pub cmd: Port<u8>,
    pub imr: Port<u32>,
    pub isr: Port<u16>,
    pub tx_config: Port<u32>,
    pub rx_config: Port<u32>,
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
            rx_ptr: Port::new(io_addr + 0x38),
            cmd: Port::new(io_addr + 0x37),
            imr: Port::new(io_addr + 0x3C),
            isr: Port::new(io_addr + 0x3E),
            tx_config: Port::new(io_addr + 0x40),
            rx_config: Port::new(io_addr + 0x44),
        }
    }
}

pub struct RTL8139 {
    ports: Ports,
    eth_addr: Option<EthernetAddress>,
    rx_buffer: Box<Vec<u8>>,
    rx_offset: usize,
    tx_buffers: [Box<Vec<u8>>; TX_BUFFERS_COUNT],
    //tx_offset: usize,
    tx_id: usize,
}

impl RTL8139 {
    pub fn new(io_addr: u16) -> Self {
        Self {
            ports: Ports::new(io_addr),
            eth_addr: None,
            rx_buffer: Box::new(vec![0; RX_BUFFER_LEN]),
            rx_offset: 0,
            tx_buffers: [
                Box::new(vec![0; TX_BUFFER_LEN]),
                Box::new(vec![0; TX_BUFFER_LEN]),
                Box::new(vec![0; TX_BUFFER_LEN]),
                Box::new(vec![0; TX_BUFFER_LEN]),
            ],
            //tx_offset: 0,
            tx_id: 0,
        }
    }

    pub fn init(&mut self) {
        // Power on
        unsafe { self.ports.config1.write(0x00 as u8) }

        // Software reset
        unsafe {
            self.ports.cmd.write(0x10 as u8);
            while self.ports.cmd.read() & 0x10 != 0 {}
        }

        // Enable Receive and Transmitter
        unsafe { self.ports.cmd.write(0x0C as u8) }

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

        // Set IMR + ISR
        unsafe { self.ports.imr.write(0x0005) }

        // Configuring Receive buffer (RCR)
        unsafe { self.ports.rx_config.write((0xF | (1 << 7)) as u32) }

        // Configuring Transmit buffer (TCR)
        unsafe { self.ports.tx_config.write(TCR_IFG | TCR_MXDMA0 | TCR_MXDMA1 | TCR_MXDMA2); }
    }
}

impl<'a> Device<'a> for RTL8139 {
    type RxToken = RxToken<'a>;
    type TxToken = TxToken<'a>;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1536;
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
        if (cmd & RX_BUFFER_EMPTY) == RX_BUFFER_EMPTY {
            return None;
        }
        print!("------------------------------------------------------------------\n");
        let uptime = kernel::clock::clock_monotonic();
        print!("[{:.6}] NET RTL8139 receiving frame:\n\n", uptime);
        print!("Command Register: 0x{:02X}\n", cmd);
        let offset = self.rx_offset;
        let header = u16::from_le_bytes(self.rx_buffer[(offset + 0)..(offset + 2)].try_into().unwrap());
        print!("Header: 0x{:04X}\n", header);
        if header & ISR_ROK != ISR_ROK {
            return None;
        }
        let length = u16::from_le_bytes(self.rx_buffer[(offset + 2)..(offset + 4)].try_into().unwrap());
        let n = length as usize;
        print!("Length: {} bytes\n", n - 4);
        let crc = u32::from_le_bytes(self.rx_buffer[(offset + n)..(offset + n + 4)].try_into().unwrap());
        print!("CRC: 0x{:08X}\n", crc);

        // Update buffer read pointer
        self.rx_offset = (offset + n + 4 + 3) & !3;
        unsafe { self.ports.rx_ptr.write((self.rx_offset - 16) as u16); }
        self.rx_offset %= RX_BUFFER_LEN;

        user::hex::print_hex(&self.rx_buffer[(offset + 4)..(offset + n)]);
        let rx = RxToken { frame: &mut self.rx_buffer[(offset + 4)..(offset + n)] };

        print!("TX Buffer: {}\n", self.tx_id);
        let port = self.ports.tx_cmds[self.tx_id].clone();
        let buffer = &mut self.tx_buffers[self.tx_id];
        let tx = TxToken { port, buffer };

        Some((rx, tx))
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        print!("------------------------------------------------------------------\n");
        let uptime = kernel::clock::clock_monotonic();
        print!("[{:.6}] NET RTL8139 transmitting frame:\n\n", uptime);
        print!("TX Buffer: {}\n", self.tx_id);
        self.tx_id = (self.tx_id + 1) % TX_BUFFERS_COUNT;
        let port = self.ports.tx_cmds[self.tx_id].clone();
        let buffer = &mut self.tx_buffers[self.tx_id];
        Some(TxToken { port, buffer })
    }
}

#[doc(hidden)]
pub struct RxToken<'a> {
    frame: &'a mut [u8]
}

impl<'a> phy::RxToken for RxToken<'a> {
    fn consume<R, F>(self, _timestamp: Instant, f: F) -> Result<R>
        where F: FnOnce(&mut [u8]) -> Result<R>
    {
        f(self.frame)
    }
}

#[doc(hidden)]
pub struct TxToken<'a> {
    port: Port<u32>,
    buffer: &'a mut [u8]
}

impl<'a> phy::TxToken for TxToken<'a> {
    fn consume<R, F>(mut self, _timestamp: Instant, len: usize, f: F) -> Result<R>
        where F: FnOnce(&mut [u8]) -> Result<R>
    {
        let res = f(&mut self.buffer[0..len]);

        user::hex::print_hex(&self.buffer[0..len]);
        if res.is_ok() {
            unsafe { self.port.write(len as u32); }
        }

        res
    }
}

pub fn init() {
    if let Some(mut pci_device) = kernel::pci::find_device(0x10EC, 0x8139) {
        pci_device.enable_bus_mastering();
        let irq = pci_device.interrupt_line;
        kernel::idt::set_irq_handler(irq, interrupt_handler);
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
    }
}

pub fn interrupt_handler() {
    print!("RTL8139 interrupt!");
    if let Some(ref mut iface) = *IFACE.lock() {
        unsafe { iface.device_mut().ports.isr.write(0x1) } // Clear the interrupt
    }
}
