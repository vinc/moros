use crate::sys;

pub fn sleep(seconds: f64) {
    unsafe { asm!("sti") }; // Restore interrupts
    printk!("DEBUG: sleep({:.2})\n", seconds);
    sys::time::sleep(seconds);
    unsafe { asm!("cli") }; // Disable interrupts
}

pub fn uptime() -> f64 {
    sys::clock::uptime()
}

pub fn realtime() -> f64 {
    sys::clock::realtime()
}
