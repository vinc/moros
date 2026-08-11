use super::boot;
use super::timer;

use crate::sys;

/// Sleeps for the specified number of seconds.
///
/// This function works by repeatedly halting the CPU until the time is
/// elapsed.
pub fn sleep(seconds: f64) {
    let start = boot::boot_time();
    while boot::boot_time() - start < seconds {
        sys::x86::hlt();
    }
}

/// Waits for the specified number of nanoseconds.
///
/// This function use a busy-wait loop with the `RDTSC` and `PAUSE`
/// instructions.
pub fn wait(nanoseconds: u64) {
    let delta = nanoseconds * timer::tsc_frequency() / 1_000_000_000;
    let start = timer::tsc();
    while timer::tsc() - start < delta {
        core::hint::spin_loop();
    }
}
