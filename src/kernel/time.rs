use crate::kernel;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref TICKS: Mutex<usize> = Mutex::new(0);
}

pub fn ticks() -> usize {
    *TICKS.lock()
}

pub fn sleep(duration: f64) {
    let interval = 1.0 / (1.193182 * 1000000.0 / 65536.0);
    let start = kernel::clock::uptime();
    while kernel::clock::uptime() - start < duration - interval {
        halt();
    }
}

pub fn halt() {
    x86_64::instructions::hlt();
}

pub fn init() {
    kernel::idt::set_irq_handler(0, interrupt_handler);
}

pub fn interrupt_handler() {
    let mut ticks = TICKS.lock();
    *ticks += 1;
}
