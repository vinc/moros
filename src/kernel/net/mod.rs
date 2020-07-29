use lazy_static::lazy_static;
use spin::Mutex;

pub mod rtl8139;
pub mod pcnet;

pub type EthernetInterface<T> = smoltcp::iface::EthernetInterface<'static, 'static, 'static, T>;

// TODO: Support dyn EthernetInterface

#[cfg(feature = "rtl8139")]
lazy_static! {
    pub static ref IFACE: Mutex<Option<EthernetInterface<rtl8139::RTL8139>>> = Mutex::new(None);
}

#[cfg(feature = "pcnet")]
lazy_static! {
    pub static ref IFACE: Mutex<Option<EthernetInterface<pcnet::PCNET>>> = Mutex::new(None);
}
