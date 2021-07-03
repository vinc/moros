use crate::kernel;

pub fn sleep(seconds: f64) {
    unsafe { asm!("sti") }; // Restore interrupts
    kernel::time::sleep(seconds);
    unsafe { asm!("cli") }; // Disable interrupts
}

pub fn uptime() -> f64 {
    kernel::clock::uptime()
}

pub fn realtime() -> f64 {
    kernel::clock::realtime()
}
