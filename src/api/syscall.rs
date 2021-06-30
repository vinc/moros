use crate::syscall;
use crate::kernel::syscall::number::*;

pub fn sleep(seconds: f64) {
    unsafe {
        syscall!(SLEEP, seconds.to_bits());
    }
}
