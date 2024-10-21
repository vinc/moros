use super::boot;
use super::timer;

use x86_64::instructions::interrupts;

pub fn halt() {
    let disabled = !interrupts::are_enabled();
    interrupts::enable_and_hlt();
    if disabled {
        interrupts::disable();
    }
}

pub fn sleep(seconds: f64) {
    let start = boot::boot_time();
    while boot::boot_time() - start < seconds {
        halt();
    }
}

pub fn nanowait(nanoseconds: u64) {
    let start = timer::tsc();
    let freq = timer::tsc_frequency();
    let delta = nanoseconds * freq;
    while timer::tsc() - start < delta {
        core::hint::spin_loop();
    }
}
