use crate::sys;

pub mod tcp;
pub mod udp;

use alloc::vec;
use lazy_static::lazy_static;
use smoltcp::iface::SocketSet;
use smoltcp::time::Duration;
use spin::Mutex;

lazy_static! {
    pub static ref SOCKETS: Mutex<SocketSet<'static>> = {
        Mutex::new(SocketSet::new(vec![]))
    };
}

fn random_port() -> u16 {
    49152 + sys::rng::get_u16() % 16384
}

fn wait(duration: Duration) {
    sys::clk::sleep((duration.total_micros() as f64) / 1000000.0);
}
