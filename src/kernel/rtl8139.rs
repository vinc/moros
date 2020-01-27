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
const ISR_ROK: u16 = 0x01;

lazy_static! {
    pub static ref IFACE: Mutex<Option<EthernetInterface<'static, 'static, 'static, RTL8139>>> = Mutex::new(None);
}

pub struct Ports {
    pub idr: [Port<u8>; 6],
    pub config1: Port<u8>,
    pub rbstart: Port<u32>,
    pub rx_ptr: Port<u16>,
    pub cmd: Port<u8>,
    pub imr: Port<u32>,
    pub isr: Port<u16>,
    pub rcr: Port<u32>,
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
            config1: Port::new(io_addr + 0x52),
            rbstart: Port::new(io_addr + 0x30),
            rx_ptr: Port::new(io_addr + 0x38),
            cmd: Port::new(io_addr + 0x37),
            imr: Port::new(io_addr + 0x3C),
            isr: Port::new(io_addr + 0x3E),
            rcr: Port::new(io_addr + 0x44),
        }
    }
}

pub struct RTL8139 {
    ports: Ports,
    eth_addr: Option<EthernetAddress>,
    rx_buffer: Box<Vec<u8>>,
    tx_buffer: Box<Vec<u8>>,
    rx_offset: usize,
    //tx_offset: usize,
}

impl RTL8139 {
    pub fn new(io_addr: u16) -> Self {
        Self {
            ports: Ports::new(io_addr),
            eth_addr: None,
            rx_buffer: Box::new(vec![0; RX_BUFFER_LEN]),
            tx_buffer: Box::new(vec![0; RX_BUFFER_LEN]), // FIXME
            rx_offset: 0,
            //tx_offset: 0,
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
        unsafe { self.ports.rbstart.write(rx_addr as u32) }

        // Set IMR + ISR
        unsafe { self.ports.imr.write(0x0005) }

        // Configuring Receive buffer
        unsafe { self.ports.rcr.write((0xF | (1 << 7)) as u32) }
    }
}

impl<'a> Device<'a> for RTL8139 {
    type RxToken = RxToken;
    type TxToken = TxToken;

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
        print!("`-> RTL8139 received data:\n\n");
        print!("cmd: 0x{:02X}\n", cmd);
        let offset = self.rx_offset;
        let header = u16::from_le_bytes(self.rx_buffer[(offset + 0)..(offset + 2)].try_into().unwrap());
        print!("header: 0x{:04X}\n", header);
        if header & ISR_ROK != ISR_ROK {
            return None;
        }
        let length = u16::from_le_bytes(self.rx_buffer[(offset + 2)..(offset + 4)].try_into().unwrap());
        let n = length as usize;
        print!("length: {}\n", n - 4);
        let data = &self.rx_buffer[(offset + 4)..(offset + n)];
        let crc = u32::from_le_bytes(self.rx_buffer[(offset + n)..(offset + n + 4)].try_into().unwrap());
        print!("crc: 0x{:08X}\n", crc);
        self.rx_offset = (offset + n + 4 + 3) & !3;
        unsafe { self.ports.rx_ptr.write((self.rx_offset - 16) as u16); }
        self.rx_offset %= RX_BUFFER_LEN;
        user::hex::print_hex(&data);
        let rx = RxToken { buffer: Box::new(data.to_vec()) };
        let tx = TxToken { buffer: self.tx_buffer.clone() };
        Some((rx, tx))
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(TxToken { buffer: self.tx_buffer.clone() })
    }
}

#[doc(hidden)]
pub struct RxToken {
    buffer: Box<Vec<u8>>,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> Result<R>
        where F: FnOnce(&mut [u8]) -> Result<R>
    {
        // TODO
        f(&mut self.buffer)
    }
}

#[doc(hidden)]
pub struct TxToken {
    buffer: Box<Vec<u8>>,
}

impl phy::TxToken for TxToken {
    fn consume<R, F>(mut self, _timestamp: Instant, _len: usize, f: F) -> Result<R>
        where F: FnOnce(&mut [u8]) -> Result<R>
    {
        // TODO
        f(&mut self.buffer)
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
            let mut routes = Routes::new(BTreeMap::new());
            let ip_addrs = [IpCidr::new(Ipv4Address::new(10, 0, 2, 15).into(), 24)];
            routes.add_default_ipv4_route(Ipv4Address::new(10, 0, 2, 2)).unwrap();
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
