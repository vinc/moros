use crate::api::fs;
use alloc::string::String;

pub const DATE_TIME_ZONE: &str = "%Y-%m-%d %H:%M:%S %z";
pub const DATE_TIME: &str = "%Y-%m-%d %H:%M:%S";
pub const DATE: &str = "%Y-%m-%d";

pub const DATE_TIME_ZONE_LEN: usize = 25;
pub const DATE_TIME_LEN: usize = 19;
pub const DATE_LEN: usize = 10;

fn read_float(path: &str) -> f64 {
    if let Ok(bytes) = fs::read_to_bytes(path) {
        if let Ok(s) = String::from_utf8(bytes) {
            if let Ok(n) = s.parse() {
                return n;
            }
        }
    }
    0.0
}

pub fn boot_time() -> f64 {
    read_float("/dev/clk/boot")
}

pub fn epoch_time() -> f64 {
    read_float("/dev/clk/epoch")
}
