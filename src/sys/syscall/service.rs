use crate::sys;

pub fn sleep(seconds: f64) {
    unsafe { asm!("sti") }; // Restore interrupts
    sys::time::sleep(seconds);
    unsafe { asm!("cli") }; // Disable interrupts
}

pub fn uptime() -> f64 {
    sys::clock::uptime()
}

pub fn realtime() -> f64 {
    sys::clock::realtime()
}

// A syscall to display something very simple from userspace
pub fn test(i: usize) -> usize {
    print!("{}\n", i);
    i + 1
}
