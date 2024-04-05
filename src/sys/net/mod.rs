mod nic;
pub mod socket;

use crate::{sys, usr};
use crate::sys::pci::DeviceConfig;

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use smoltcp::iface::Interface;
use smoltcp::phy::DeviceCapabilities;
use smoltcp::time::Instant;
use smoltcp::wire::EthernetAddress;
use spin::Mutex;

pub static NET: Mutex<Option<(Interface, EthernetDevice)>> = Mutex::new(None);

#[repr(u8)]
pub enum SocketStatus {
    IsListening = 0,
    IsActive = 1,
    IsOpen = 2,
    CanSend = 3,
    MaySend = 4,
    CanRecv = 5,
    MayRecv = 6,
}

fn time() -> Instant {
    Instant::from_micros((sys::clock::realtime() * 1000000.0) as i64)
}

#[derive(Clone)]
pub enum EthernetDevice {
    RTL8139(nic::rtl8139::Device),
    PCNET(nic::pcnet::Device),
    E1000(nic::e1000::Device),
    //VirtIO,
}

pub trait EthernetDeviceIO {
    fn config(&self) -> Arc<Config>;
    fn stats(&self) -> Arc<Stats>;
    fn receive_packet(&mut self) -> Option<Vec<u8>>;
    fn transmit_packet(&mut self, len: usize);
    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8];
}

impl EthernetDeviceIO for EthernetDevice {
    fn config(&self) -> Arc<Config> {
        match self {
            EthernetDevice::RTL8139(dev) => dev.config(),
            EthernetDevice::PCNET(dev) => dev.config(),
            EthernetDevice::E1000(dev) => dev.config(),
        }
    }

    fn stats(&self) -> Arc<Stats> {
        match self {
            EthernetDevice::RTL8139(dev) => dev.stats(),
            EthernetDevice::PCNET(dev) => dev.stats(),
            EthernetDevice::E1000(dev) => dev.stats(),
        }
    }

    fn receive_packet(&mut self) -> Option<Vec<u8>> {
        match self {
            EthernetDevice::RTL8139(dev) => dev.receive_packet(),
            EthernetDevice::PCNET(dev) => dev.receive_packet(),
            EthernetDevice::E1000(dev) => dev.receive_packet(),
        }
    }

    fn transmit_packet(&mut self, len: usize) {
        match self {
            EthernetDevice::RTL8139(dev) => dev.transmit_packet(len),
            EthernetDevice::PCNET(dev) => dev.transmit_packet(len),
            EthernetDevice::E1000(dev) => dev.transmit_packet(len),
        }
    }

    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8] {
        match self {
            EthernetDevice::RTL8139(dev) => dev.next_tx_buffer(len),
            EthernetDevice::PCNET(dev) => dev.next_tx_buffer(len),
            EthernetDevice::E1000(dev) => dev.next_tx_buffer(len),
        }
    }
}

impl<'a> smoltcp::phy::Device for EthernetDevice {
    type RxToken<'b> = RxToken where Self: 'b;
    type TxToken<'b> = TxToken where Self: 'b;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(64);
        caps
    }

    fn receive(
        &mut self,
        _instant: smoltcp::time::Instant,
    ) -> Option<(Self::RxToken<'a>, Self::TxToken<'a>)> {
        if let Some(buffer) = self.receive_packet() {
            if self.config().is_debug_enabled() {
                debug!("NET Packet Received");
                usr::hex::print_hex(&buffer);
            }
            self.stats().rx_add(buffer.len() as u64);
            let rx = RxToken { buffer };
            let tx = TxToken {
                device: self.clone(),
            };
            Some((rx, tx))
        } else {
            None
        }
    }

    fn transmit(
        &mut self,
        _instant: smoltcp::time::Instant
    ) -> Option<Self::TxToken<'a>> {
        let tx = TxToken {
            device: self.clone(),
        };
        Some(tx)
    }
}

#[doc(hidden)]
pub struct RxToken {
    buffer: Vec<u8>,
}

impl smoltcp::phy::RxToken for RxToken {
    fn consume<R, F>(mut self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        f(&mut self.buffer)
    }
}

#[doc(hidden)]
pub struct TxToken {
    device: EthernetDevice,
}
impl smoltcp::phy::TxToken for TxToken {
    fn consume<R, F>(mut self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let config = self.device.config();
        let buf = self.device.next_tx_buffer(len);
        let res = f(buf);
        if config.is_debug_enabled() {
            debug!("NET Packet Transmitted");
            usr::hex::print_hex(buf);
        }
        self.device.transmit_packet(len);
        self.device.stats().tx_add(len as u64);
        res
    }
}

pub struct Config {
    debug: AtomicBool,
    mac: Mutex<Option<EthernetAddress>>,
}

impl Config {
    fn new() -> Self {
        Self {
            debug: AtomicBool::new(false),
            mac: Mutex::new(None),
        }
    }

    fn is_debug_enabled(&self) -> bool {
        self.debug.load(Ordering::Relaxed)
    }

    pub fn enable_debug(&self) {
        self.debug.store(true, Ordering::Relaxed);
    }

    pub fn disable_debug(&self) {
        self.debug.store(false, Ordering::Relaxed)
    }

    fn mac(&self) -> Option<EthernetAddress> {
        *self.mac.lock()
    }

    fn update_mac(&self, mac: EthernetAddress) {
        *self.mac.lock() = Some(mac);
    }
}

pub struct Stats {
    rx_bytes_count: AtomicU64,
    tx_bytes_count: AtomicU64,
    rx_packets_count: AtomicU64,
    tx_packets_count: AtomicU64,
}

impl Stats {
    fn new() -> Self {
        Self {
            rx_bytes_count: AtomicU64::new(0),
            tx_bytes_count: AtomicU64::new(0),
            rx_packets_count: AtomicU64::new(0),
            tx_packets_count: AtomicU64::new(0),
        }
    }

    pub fn rx_bytes_count(&self) -> u64 {
        self.rx_bytes_count.load(Ordering::Relaxed)
    }

    pub fn tx_bytes_count(&self) -> u64 {
        self.tx_bytes_count.load(Ordering::Relaxed)
    }

    pub fn rx_packets_count(&self) -> u64 {
        self.rx_packets_count.load(Ordering::Relaxed)
    }

    pub fn tx_packets_count(&self) -> u64 {
        self.tx_packets_count.load(Ordering::Relaxed)
    }

    pub fn rx_add(&self, bytes_count: u64) {
        self.rx_packets_count.fetch_add(1, Ordering::SeqCst);
        self.rx_bytes_count.fetch_add(bytes_count, Ordering::SeqCst);
    }

    pub fn tx_add(&self, bytes_count: u64) {
        self.tx_packets_count.fetch_add(1, Ordering::SeqCst);
        self.tx_bytes_count.fetch_add(bytes_count, Ordering::SeqCst);
    }
}

fn find_device(vendor_id: u16, device_id: u16) -> Option<DeviceConfig> {
    if let Some(mut dev) = sys::pci::find_device(vendor_id, device_id) {
        dev.enable_bus_mastering();
        Some(dev)
    } else {
        None
    }
}

const E1000_DEVICES: [u16; 8] = [
    0x1004, // 82543GC (Intel PRO/1000 T)
    0x100C, // 82544GC (Intel PRO/1000 T)
    0x100E, // 82540EM (Intel PRO/1000 MT)
    0x100F, // 82545EM (Intel PRO/1000 MT)
    0x107C, // 82541PI (Intel PRO/1000 GT)
    0x10D3, // 82574L
    0x10F5, // 82567LM
    0x153A, // I217-LM
];

pub fn init() {
    let add = |mut device: EthernetDevice, name| {
        if let Some(mac) = device.config().mac() {
            log!("NET {} MAC {}", name, mac);

            let config = smoltcp::iface::Config::new(mac.into());
            let iface = Interface::new(config, &mut device, time());

            *NET.lock() = Some((iface, device));
        }
    };
    if let Some(dev) = find_device(0x10EC, 0x8139) {
        let io = dev.io_base();
        let nic = nic::rtl8139::Device::new(io);
        add(EthernetDevice::RTL8139(nic), "RTL8139");
    }
    if let Some(dev) = find_device(0x1022, 0x2000) {
        let io = dev.io_base();
        let nic = nic::pcnet::Device::new(io);
        add(EthernetDevice::PCNET(nic), "PCNET");
    }
    for id in E1000_DEVICES {
        if let Some(dev) = find_device(0x8086, id) {
            let io = dev.io_base();
            let mem = dev.mem_base();
            let bar = dev.bar_type();
            let nic = nic::e1000::Device::new(io, mem, bar);
            add(EthernetDevice::E1000(nic), "E1000");
        }
    }
}
