use crate::{sys, usr};

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use smoltcp::iface::{InterfaceBuilder, NeighborCache, Routes};
use smoltcp::phy::DeviceCapabilities;
use smoltcp::phy::{Device, Medium};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpCidr, Ipv4Address};
use spin::Mutex;

mod rtl8139;
mod pcnet;

pub type Interface = smoltcp::iface::Interface<'static, EthernetDevice>;

lazy_static! {
    pub static ref IFACE: Mutex<Option<Interface>> = Mutex::new(None);
}

#[derive(Clone)]
pub enum EthernetDevice {
    RTL8139(rtl8139::Device),
    PCNET(pcnet::Device),
    //E2000,
    //VirtIO,
}

pub trait EthernetDeviceIO {
    fn init(&mut self);
    fn stats(&self) -> Stats;
    fn mac(&self) -> Option<EthernetAddress>;
    fn receive_packet(&mut self) -> Option<Vec<u8>>;
    fn transmit_packet(&mut self, len: usize);
    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8];
}

impl EthernetDeviceIO for EthernetDevice {
    fn init(&mut self) {
        match self {
            EthernetDevice::RTL8139(dev) => dev.init(),
            EthernetDevice::PCNET(dev) => dev.init(),
        }
    }

    fn stats(&self) -> Stats {
        match self {
            EthernetDevice::RTL8139(dev) => dev.stats(),
            EthernetDevice::PCNET(dev) => dev.stats(),
        }
    }

    fn mac(&self) -> Option<EthernetAddress> {
        match self {
            EthernetDevice::RTL8139(dev) => dev.mac(),
            EthernetDevice::PCNET(dev) => dev.mac(),
        }
    }

    fn receive_packet(&mut self) -> Option<Vec<u8>> {
        match self {
            EthernetDevice::RTL8139(dev) => dev.receive_packet(),
            EthernetDevice::PCNET(dev) => dev.receive_packet(),
        }
    }

    fn transmit_packet(&mut self, len: usize) {
        match self {
            EthernetDevice::RTL8139(dev) => dev.transmit_packet(len),
            EthernetDevice::PCNET(dev) => dev.transmit_packet(len),
        }
    }

    fn next_tx_buffer(&mut self, len: usize) -> &mut [u8] {
        match self {
            EthernetDevice::RTL8139(dev) => dev.next_tx_buffer(len),
            EthernetDevice::PCNET(dev) => dev.next_tx_buffer(len),
        }
    }
}

impl<'a> smoltcp::phy::Device<'a> for EthernetDevice {
    type RxToken = RxToken;
    type TxToken = TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1500;
        caps.max_burst_size = Some(1);
        caps
    }

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        if let Some(buffer) = self.receive_packet() {
            self.stats().rx_add(buffer.len() as u64);
            let rx = RxToken { buffer };
            let tx = TxToken { device: self.clone() };
            Some((rx, tx))
        } else {
            None
        }
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        let tx = TxToken { device: self.clone() };
        Some(tx)
    }
}

#[doc(hidden)]
pub struct RxToken {
    buffer: Vec<u8>,
}

impl smoltcp::phy::RxToken for RxToken {
     fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> smoltcp::Result<R> where F: FnOnce(&mut [u8]) -> smoltcp::Result<R> {
        debug!("RxToken#consume");
        usr::hex::print_hex(&self.buffer);

        f(&mut self.buffer)
    }
}

#[doc(hidden)]
pub struct TxToken {
    device: EthernetDevice,
}
impl smoltcp::phy::TxToken for TxToken {
    fn consume<R, F>(mut self, _timestamp: Instant, len: usize, f: F) -> smoltcp::Result<R> where F: FnOnce(&mut [u8]) -> smoltcp::Result<R> {
        let mut buf = self.device.next_tx_buffer(len);
        let res = f(&mut buf);

        debug!("TxToken#consume");
        usr::hex::print_hex(&buf.to_vec());

        if res.is_ok() {
            self.device.transmit_packet(len);
            self.device.stats().tx_add(len as u64);
        }
        /*
        if self.device.debug_mode {
            usr::hex::print_hex(&buf);
        }
        */

        res
    }
}

struct InnerStats {
    rx_bytes_count: AtomicU64,
    tx_bytes_count: AtomicU64,
    rx_packets_count: AtomicU64,
    tx_packets_count: AtomicU64,
}

impl InnerStats {
    fn new() -> Self {
        Self {
            rx_bytes_count: AtomicU64::new(0),
            tx_bytes_count: AtomicU64::new(0),
            rx_packets_count: AtomicU64::new(0),
            tx_packets_count: AtomicU64::new(0),
        }
    }
}

#[derive(Clone)]
pub struct Stats {
    stats: Arc<InnerStats>
}

impl Stats {
    fn new() -> Self {
        Self {
            stats: Arc::new(InnerStats::new())
        }
    }

    pub fn rx_bytes_count(&self) -> u64 {
        self.stats.rx_bytes_count.load(Ordering::Relaxed)
    }

    pub fn tx_bytes_count(&self) -> u64 {
        self.stats.tx_bytes_count.load(Ordering::Relaxed)
    }

    pub fn rx_packets_count(&self) -> u64 {
        self.stats.rx_packets_count.load(Ordering::Relaxed)
    }

    pub fn tx_packets_count(&self) -> u64 {
        self.stats.tx_packets_count.load(Ordering::Relaxed)
    }

    pub fn rx_add(&self, bytes_count: u64) {
        self.stats.rx_packets_count.fetch_add(1, Ordering::SeqCst);
        self.stats.rx_bytes_count.fetch_add(bytes_count, Ordering::SeqCst);
    }

    pub fn tx_add(&self, bytes_count: u64) {
        self.stats.tx_packets_count.fetch_add(1, Ordering::SeqCst);
        self.stats.tx_bytes_count.fetch_add(bytes_count, Ordering::SeqCst);
    }
}

fn find_pci_io_base(vendor_id: u16, device_id: u16) -> Option<u16> {
    if let Some(mut pci_device) = sys::pci::find_device(vendor_id, device_id) {
        pci_device.enable_bus_mastering();
        let io_base = (pci_device.base_addresses[0] as u16) & 0xFFF0;
        Some(io_base)
    } else {
        None
    }
}

pub fn init() {
    let add_interface = |mut device: EthernetDevice, name| {
        device.init();
        if let Some(mac) = device.mac() {
            log!("NET {} MAC {}\n", name, mac);

            let neighbor_cache = NeighborCache::new(BTreeMap::new());
            let routes = Routes::new(BTreeMap::new());
            let ip_addrs = [IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0)];

            let medium = device.capabilities().medium;
            let mut builder = InterfaceBuilder::new(device, vec![]).ip_addrs(ip_addrs).routes(routes);
            if medium == Medium::Ethernet {
                builder = builder.hardware_addr(mac.into()).neighbor_cache(neighbor_cache);
            }
            let iface = builder.finalize();

            *IFACE.lock() = Some(iface);
        }
    };

    if let Some(io_base) = find_pci_io_base(0x10EC, 0x8139) {
        add_interface(EthernetDevice::RTL8139(rtl8139::Device::new(io_base)), "RTL8139");
    }
    if let Some(io_base) = find_pci_io_base(0x1022, 0x2000) {
        add_interface(EthernetDevice::PCNET(pcnet::Device::new(io_base)), "PCNET");
    }

}
