use crate::kernel;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn print_fmt(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    SERIAL1.lock().write_fmt(args).expect("Could not print to serial");
}

pub fn init() {
    kernel::idt::set_irq_handler(4, interrupt_handler);
}

fn interrupt_handler() {
    let b = SERIAL1.lock().receive();
    let c = match b as char {
        '\r' => '\n',
        '\x7F' => '\x08', // Delete => Backspace
        c => c,
    };
    kernel::console::key_handle(c);
}
