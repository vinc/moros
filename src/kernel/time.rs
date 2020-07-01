use crate::kernel;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::kernel::cmos::CMOS;

const PIT_FREQUENCY: f64 = 1.193_182; // Mhz
const PIT_INTERVAL: f64 = 1.0 / (PIT_FREQUENCY * 1_000_000.0 / 65_536.0);

lazy_static! {
    pub static ref PIT_TICKS: Mutex<usize> = Mutex::new(0);
    pub static ref LAST_RTC_UPDATE: Mutex<usize> = Mutex::new(0);
}

pub fn ticks() -> usize {
    *PIT_TICKS.lock()
}

pub fn time_between_ticks() -> f64 {
    PIT_INTERVAL
}

pub fn last_rtc_update() -> usize {
    *LAST_RTC_UPDATE.lock()
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
    kernel::idt::set_irq_handler(0, pit_interrupt_handler);
    kernel::idt::set_irq_handler(8, rtc_interrupt_handler);
    CMOS::new().enable_update_interrupt();
}

pub fn pit_interrupt_handler() {
    let mut ticks = PIT_TICKS.lock();
    *ticks += 1;
}

pub fn rtc_interrupt_handler() {
    let ticks = PIT_TICKS.lock();
    let mut last_update = LAST_RTC_UPDATE.lock();
    *last_update = *ticks;
    CMOS::new().notify_end_of_interrupt();
}
