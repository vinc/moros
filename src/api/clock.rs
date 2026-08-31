#[cfg(target_arch = "x86_64")] // TODO: Remove
use crate::api::fs;

use alloc::string::String;

pub const DATE_TIME_ZONE: &str = "%Y-%m-%d %H:%M:%S %z";
pub const DATE_TIME: &str = "%Y-%m-%d %H:%M:%S";
pub const DATE: &str = "%Y-%m-%d";

pub const DATE_TIME_ZONE_LEN: usize = 25;
pub const DATE_TIME_LEN: usize = 19;
pub const DATE_LEN: usize = 10;

#[cfg(target_arch = "x86_64")] // TODO: Remove
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

#[cfg(target_arch = "x86_64")] // TODO: Remove
pub fn boot_time() -> f64 {
    read_float("/dev/clk/boot")
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
pub fn epoch_time() -> f64 {
    read_float("/dev/clk/epoch")
}
