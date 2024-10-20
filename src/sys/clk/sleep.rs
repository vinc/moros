use crate::sys;

use core::sync::atomic::Ordering;
use x86_64::instructions::interrupts;

pub fn halt() {
    let disabled = !interrupts::are_enabled();
    interrupts::enable_and_hlt();
    if disabled {
        interrupts::disable();
    }
}

pub fn sleep(seconds: f64) {
    let start = sys::clock::uptime();
    while sys::clock::uptime() - start < seconds {
        halt();
    }
}

pub fn nanowait(nanoseconds: u64) {
    let start = super::rdtsc();
    let clock = super::CLOCKS_PER_NANOSECOND.load(Ordering::Relaxed);
    let delta = nanoseconds * clock;
    while super::rdtsc() - start < delta {
        core::hint::spin_loop();
    }
}
