use crate::syscall;
use crate::kernel::syscall::number::*;

pub fn sleep(seconds: f64) {
    unsafe { syscall!(SLEEP, seconds.to_bits()) };
}

pub fn uptime() -> f64 {
    let res = unsafe { syscall!(UPTIME) };
    f64::from_bits(res as u64)
}
