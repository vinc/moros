use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use crate::{print, kernel};
use smoltcp::Result;
use smoltcp::phy::{Device, DeviceCapabilities};
use smoltcp::phy;
use smoltcp::time::Instant;
use smoltcp::wire::EthernetAddress;
use x86_64::instructions::port::Port;

pub struct Ports {
    pub idr: [Port<u8>; 6],
    pub config1: Port<u8>,
    pub rbstart: Port<u32>,
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
}

impl RTL8139 {
    pub fn new(io_addr: u16) -> Self {
        Self {
            ports: Ports::new(io_addr),
            eth_addr: None,
            rx_buffer: Box::new(vec![0; 8192 + 16]),
            tx_buffer: Box::new(vec![0; 8192 + 16]),
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

        let rx_addr = &self.rx_buffer[0] as *const u8;

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

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let rx = RxToken { buffer: self.rx_buffer.clone() };
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

        let io_addr = (pci_device.base_addresses[0] as u16) & 0xFFF0;

        kernel::idt::set_irq_handler(pci_device.interrupt_line, interrupt_handler);


        let mut rtl8139_device = RTL8139::new(io_addr);

        rtl8139_device.init();

        if let Some(eth_addr) = rtl8139_device.eth_addr {
            let uptime = kernel::clock::clock_monotonic();
            print!("[{:.6}] NET RTL8139 MAC {}\n", uptime, eth_addr);
        }
    }
}

pub fn interrupt_handler() {
    print!("RTL8139 interrupt!");
    if let Some(device) = kernel::pci::find_device(0x10EC, 0x8139) {
        let io_addr = (device.base_addresses[0] as u16) & 0xFFF0;
        let mut ports = Ports::new(io_addr);
        unsafe { ports.isr.write(0x1) } // Clear the interrupt
    }
}
