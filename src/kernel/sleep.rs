use crate::kernel::clock;

pub fn sleep(duration: f64) {
    let start = clock::clock_monotonic();
    while clock::clock_monotonic() - start < duration {
        halt();
    }
}

pub fn halt() {
    x86_64::instructions::hlt();
}
