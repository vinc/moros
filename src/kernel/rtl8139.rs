use crate::{print, kernel};
use x86_64::instructions::port::Port;

pub struct Ports {
    pub idr: [Port<u8>; 6],
    pub rbstart: Port<u32>,
    pub cmd: Port<u8>,
    pub imr: Port<u32>,
    pub isr: Port<u16>,
    pub config1: Port<u8>
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
            rbstart: Port::new(io_addr + 0x30),
            cmd : Port::new(io_addr + 0x37),
            imr : Port::new(io_addr + 0x3C),
            isr : Port::new(io_addr + 0x3E),
            config1 : Port::new(io_addr + 0x52),
        }
    }
}

pub fn init() {
    if let Some(device) = kernel::pci::find_device(0x10EC, 0x8139) {
        let io_addr = (device.base_addresses[0] as u16) & 0xFFF0;
        let mut ports = Ports::new(io_addr);

        // Power on
        unsafe { ports.config1.write(0x00 as u8) }

        // Software reset
        unsafe {
            ports.cmd.write(0x10 as u8);
            while ports.cmd.read() & 0x10 != 0 {}
        }

        // Enable Receive and Transmitter
        unsafe { ports.cmd.write(0x0C as u8) }

        // Read MAC addr
        let mac = [
            unsafe { ports.idr[0].read() },
            unsafe { ports.idr[1].read() },
            unsafe { ports.idr[2].read() },
            unsafe { ports.idr[3].read() },
            unsafe { ports.idr[4].read() },
            unsafe { ports.idr[5].read() },
        ];

        let uptime = kernel::clock::clock_monotonic();
        print!(
            "[{:.6}] RTL8139 MAC {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n",
            uptime, mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );
    }
}
