use super::boot;
use super::timer;

use x86_64::instructions::interrupts;

/// Halts the CPU until the next interrupt.
///
/// This function preserves interrupt state.
pub fn halt() {
    let disabled = !interrupts::are_enabled();
    interrupts::enable_and_hlt();
    if disabled {
        interrupts::disable();
    }
}

/// Sleeps for the specified number of seconds.
///
/// This function works by repeatedly halting the CPU until the time is
/// elapsed.
pub fn sleep(seconds: f64) {
    let start = boot::boot_time();
    while boot::boot_time() - start < seconds {
        halt();
    }
}

/// Waits for the specified number of nanoseconds.
///
/// This function use a busy-wait loop with the `RDTSC` and `PAUSE`
/// instructions.
pub fn wait(nanoseconds: u64) {
    let delta = nanoseconds * timer::tsc_frequency();
    let start = timer::tsc();
    while timer::tsc() - start < delta {
        core::hint::spin_loop();
    }
}
