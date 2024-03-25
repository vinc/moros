use crate::sys::allocator::PhysBuf;
use crate::sys::net::{EthernetDeviceIO, Config, Stats};

use alloc::sync::Arc;
use alloc::vec::Vec;
use smoltcp::wire::EthernetAddress;
use x86_64::instructions::port::Port;

// https://ozlabs.org/~rusty/virtio-spec/virtio-0.9.5.pdf
// https://docs.oasis-open.org/virtio/virtio/v1.0/virtio-v1.0.html

const ACKNOWLEDGE: u8 = 1;
const DRIVER: u8 = 2;
const DRIVER_OK: u8 = 4;
const FAILED: u8 = 128;

#[derive(Clone)]
pub struct Ports {
    pub device_features: Port<u32>, // r
    pub driver_features: Port<u32>, // r+w
    pub queue_addr: Port<u32>,      // r+w
    pub queue_size: Port<u16>,      // r
    pub queue_select: Port<u16>,    // r+w
    pub queue_notify: Port<u16>,    // r+w
    pub device_status: Port<u8>,    // r+w
    pub isr_status: Port<u8>,       // r
    pub mac: [Port<u8>; 6],
}

impl Ports {
    pub fn new(io_base: u16) -> Self {
        Self {
            device_features: Port::new(io_base + 0x00),
            driver_features: Port::new(io_base + 0x04),
            queue_addr: Port::new(io_base + 0x08),
            queue_size: Port::new(io_base + 0x0C),
            queue_select: Port::new(io_base + 0x0E),
            queue_notify: Port::new(io_base + 0x10),
            device_status: Port::new(io_base + 0x12),
            isr_status: Port::new(io_base + 0x13),
            mac: [
                Port::new(io_base + 0x14),
                Port::new(io_base + 0x15),
                Port::new(io_base + 0x16),
                Port::new(io_base + 0x17),
                Port::new(io_base + 0x18),
                Port::new(io_base + 0x19),
            ],
        }
    }

    fn mac(&mut self) -> [u8; 6] {
        unsafe {
            [
                self.mac[0].read(),
                self.mac[1].read(),
                self.mac[2].read(),
                self.mac[3].read(),
                self.mac[4].read(),
                self.mac[5].read(),
            ]
        }
    }
}

#[derive(Clone)]
pub struct Device {
    config: Arc<Config>,
    stats: Arc<Stats>,
    ports: Ports,

    rx_buffer: PhysBuf,
    tx_buffer: PhysBuf,
}

impl Device {
    pub fn new(io_base: u16) -> Self {
        let mut device = Self {
            ports: Ports::new(io_base),
            config: Arc::new(Config::new()),
            stats: Arc::new(Stats::new()),
            rx_buffer: PhysBuf::new((4096 * 5) / 8),
            tx_buffer: PhysBuf::new((4096 * 5) / 8),
        };
        device.init();
        device
    }

    fn init(&mut self) {
        unsafe {
            self.ports.device_status.write(0); // Reset
            
            let device_status = self.ports.device_status.read();
            self.ports.device_status.write(device_status | ACKNOWLEDGE);

            let device_status = self.ports.device_status.read();
            self.ports.device_status.write(device_status | DRIVER);
        }

        let device_features = unsafe { self.ports.device_features.read() };
        debug!("VirtIO Net: device features: {:#X}", device_features);


        let device_status = unsafe { self.ports.device_status.read() };
        debug!("VirtIO Net: device status: {:#X}", device_status);

        // RX
        unsafe {
            let queue_index = 0;
            self.ports.queue_select.write(queue_index);
            let queue_size = self.ports.queue_size.read() as usize;
            debug!("VirtIO Net: queue({}) size: {}", queue_index, queue_size);
        }

        // TX
        unsafe {
            let queue_index = 1;
            self.ports.queue_select.write(queue_index);
            let queue_size = self.ports.queue_size.read() as usize;
            debug!("VirtIO Net: queue({}) size: {}", queue_index, queue_size);
        };

        self.config.update_mac(EthernetAddress::from_bytes(&self.ports.mac()));

        unsafe {
            let device_status = self.ports.device_status.read();
            self.ports.device_status.write(device_status | DRIVER_OK);
        }
    }

    fn read(&self, addr: u16) -> u32 {
        debug!("READ: {:#X}", addr);
        0
    }

    fn write(&self, addr: u16, data: u32) {
        debug!("WRITE: {:#X}", addr);
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
        debug!("RECV");
        None
    }

    fn transmit_packet(&mut self, len: usize) {
        debug!("SEND");
    }

    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8] {
        &mut self.tx_buffer[0..len] // FIXME
    }
}
