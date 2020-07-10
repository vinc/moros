use crate::kernel;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
use core::fmt;
use core::fmt::Write;

lazy_static! {
    pub static ref SERIAL: Mutex<Serial> = Mutex::new(Serial::new(0x3F8));
}

pub struct Serial {
    pub port: SerialPort
}

impl Serial {
    fn new(addr: u16) -> Self {
        let mut port = unsafe { SerialPort::new(addr) };
        port.init();
        Self { port }
    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.port.send(byte);
    }
}

impl fmt::Write for Serial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn print_fmt(args: fmt::Arguments) {
    SERIAL.lock().write_fmt(args).expect("Could not print to serial");
}

pub fn init() {
    kernel::idt::set_irq_handler(4, interrupt_handler);
}

fn interrupt_handler() {
    let b = SERIAL.lock().port.receive();
    let c = match b as char {
        '\r' => '\n',
        '\x7F' => '\x08', // Delete => Backspace
        c => c,
    };
    kernel::console::key_handle(c);
}
