use crate::kernel;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;

lazy_static! {
    pub static ref TICKS: Mutex<usize> = Mutex::new(0);
}

pub fn ticks() -> usize {
    *TICKS.lock()
}

pub fn sleep(duration: f64) {
    let start = kernel::clock::clock_monotonic();
    while kernel::clock::clock_monotonic() - start < duration {
        halt();
    }
}

pub fn halt() {
    x86_64::instructions::hlt();
}

pub extern "x86-interrupt" fn interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let mut ticks = TICKS.lock();
    *ticks += 1;

    unsafe {
        kernel::pic::PICS.lock().notify_end_of_interrupt(kernel::idt::IRQ0);
    }
}
