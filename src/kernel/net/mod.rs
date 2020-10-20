use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

// TODO: Support dyn EthernetInterface
pub type EthernetInterface<T> = smoltcp::iface::EthernetInterface<'static, 'static, 'static, T>;

#[cfg(feature = "rtl8139")]
pub mod rtl8139;

#[cfg(feature = "rtl8139")]
lazy_static! {
    pub static ref IFACE: Mutex<Option<EthernetInterface<rtl8139::RTL8139>>> = Mutex::new(None);
}

#[cfg(feature = "rtl8139")]
pub fn init() {
    rtl8139::init();
}

#[cfg(feature = "pcnet")]
pub mod pcnet;

#[cfg(feature = "pcnet")]
lazy_static! {
    pub static ref IFACE: Mutex<Option<EthernetInterface<pcnet::PCNET>>> = Mutex::new(None);
}

#[cfg(feature = "pcnet")]
pub fn init() {
    pcnet::init();
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
