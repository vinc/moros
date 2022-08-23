use crate::api::fs;
use core::convert::TryInto;

pub const DATE_TIME_ZONE: &str = "%Y-%m-%d %H:%M:%S %z";
pub const DATE_TIME: &str = "%Y-%m-%d %H:%M:%S";
pub const DATE: &str = "%Y-%m-%d";

fn read_float(path: &str) -> f64 {
    if let Ok(bytes) = fs::read_to_bytes(path) {
        if bytes.len() == 8 {
            return f64::from_be_bytes(bytes[0..8].try_into().unwrap());
        }
    }

    return 0.0;
}

pub fn uptime() -> f64 {
    read_float("/dev/clk/uptime")
}

pub fn realtime() -> f64 {
    read_float("/dev/clk/realtime")
}
