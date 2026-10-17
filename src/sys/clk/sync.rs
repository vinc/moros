use super::timer;

use crate::sys;

// Convert to the nearest number of ticks
fn seconds_to_ticks(seconds: f64) -> usize {
    (seconds / timer::time_between_ticks() + 0.5) as usize
}

/// Sleeps for the specified number of seconds.
///
/// This function works by repeatedly halting the CPU until the time is
/// elapsed.
pub fn sleep(seconds: f64) {
    let count = seconds_to_ticks(seconds);
    let start = timer::ticks();
    while timer::ticks() - start < count {
        sys::x86::hlt();
    }
}

/// Waits for the specified number of nanoseconds.
///
/// This function use a busy-wait loop with the `RDTSC` and `PAUSE`
/// instructions.
pub fn wait(nanoseconds: u64) {
    let timer = timer::Timer::new(nanoseconds);
    while !timer.is_elapsed() {
        core::hint::spin_loop();
    }
}

#[test_case]
fn test_sleep_seconds_to_ticks() {
    assert_eq!(seconds_to_ticks(0.0000), 0);
    assert_eq!(seconds_to_ticks(0.0004), 0);
    assert_eq!(seconds_to_ticks(0.0006), 1);
    assert_eq!(seconds_to_ticks(0.0010), 1);
    assert_eq!(seconds_to_ticks(0.0014), 1);
    assert_eq!(seconds_to_ticks(0.0016), 2);
    assert_eq!(seconds_to_ticks(0.1000), 100);
    assert_eq!(seconds_to_ticks(1.0000), 1000);
}
