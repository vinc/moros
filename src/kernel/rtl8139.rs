use alloc::vec::Vec;
use crate::{print, kernel};
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

pub fn init() {
    if let Some(mut device) = kernel::pci::find_device(0x10EC, 0x8139) {
        device.enable_bus_mastering();

        let io_addr = (device.base_addresses[0] as u16) & 0xFFF0;
        let mut ports = Ports::new(io_addr);

        //print!("interrupt pin: 0x{:02X}\n", device.interrupt_pin);
        //print!("IRQ: {}\n", device.interrupt_line);
        kernel::idt::set_irq_handler(device.interrupt_line, interrupt_handler);

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
            "[{:.6}] NET RTL8139 MAC {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}\n",
            uptime, mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );

        let mut rx_buf: Vec<u8> = Vec::new();
        rx_buf.resize(8192 + 16, 0);

        let rx_addr = &rx_buf[0] as *const u8;
        //print!("rx_addr: 0x{:016X}\n", rx_addr as u64);
        //print!("rx_addr: 0x{:08X}\n", rx_addr as u32);

        // Init Receive buffer
        unsafe { ports.rbstart.write(rx_addr as u32) }

        // Set IMR + ISR
        unsafe { ports.imr.write(0x0005) }

        // Configuring Receive buffer
        unsafe { ports.rcr.write((0xF | (1 << 7)) as u32) }
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
