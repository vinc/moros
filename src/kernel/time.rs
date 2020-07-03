use core::sync::atomic::{AtomicUsize, Ordering};
use crate::kernel::cmos::CMOS;
use crate::kernel;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

// At boot the PIT starts with a frequency divider of 0 (equivalent to 65536)
// which will result in about 54.926 ms between ticks.
// During init we will change the divider to 1193 to have about 1.000 ms
// between ticks to improve time measurements accuracy.
const PIT_FREQUENCY: f64 = 3_579_545.0 / 3.0; // 1_193_181.666 Hz
const PIT_DIVIDER: usize = 1193;
const PIT_INTERVAL: f64 = (PIT_DIVIDER as f64) / PIT_FREQUENCY;

static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);
static LAST_RTC_UPDATE: AtomicUsize = AtomicUsize::new(0);

pub fn ticks() -> usize {
    PIT_TICKS.load(Ordering::Relaxed)
}

pub fn time_between_ticks() -> f64 {
    PIT_INTERVAL
}

pub fn last_rtc_update() -> usize {
    LAST_RTC_UPDATE.load(Ordering::Relaxed)
}

pub fn sleep(duration: f64) {
    let start = kernel::clock::uptime();
    while kernel::clock::uptime() - start < duration - time_between_ticks() {
        halt();
    }
}

pub fn halt() {
    x86_64::instructions::hlt();
}

pub fn init() {
    // PIT timmer
    let divider = if PIT_DIVIDER < 65536 { PIT_DIVIDER } else { 0 };
    set_pit_frequency_divider(divider as u16);
    kernel::idt::set_irq_handler(0, pit_interrupt_handler);

    // RTC timmer
    kernel::idt::set_irq_handler(8, rtc_interrupt_handler);
    CMOS::new().enable_update_interrupt();
}

/// The frequency divider must be between 0 and 65535, with 0 acting as 65536
fn set_pit_frequency_divider(divider: u16) {
    interrupts::without_interrupts(|| {
        let bytes = divider.to_le_bytes();
        let mut cmd: Port<u8> = Port::new(0x43);
        let mut data: Port<u8> = Port::new(0x40);
        unsafe {
            cmd.write(0x36);
            data.write(bytes[0]);
            data.write(bytes[1]);
        }
    });
}

pub fn pit_interrupt_handler() {
    PIT_TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn rtc_interrupt_handler() {
    LAST_RTC_UPDATE.store(ticks(), Ordering::Relaxed);
    CMOS::new().notify_end_of_interrupt();
}
