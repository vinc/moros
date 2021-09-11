use crate::sys;
use crate::sys::fs::FileIO;
use alloc::string::String;
use alloc::string::ToString;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;

lazy_static! {
    pub static ref STDIN: Mutex<String> = Mutex::new(String::new());
    pub static ref ECHO: Mutex<bool> = Mutex::new(true);
    pub static ref RAW: Mutex<bool> = Mutex::new(false);
}

#[derive(Debug, Clone)]
pub struct Console;

impl Console {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileIO for Console {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let s = if buf.len() == 1 {
            read_char().to_string()
        } else {
            read_line()
        };
        let n = s.len();
        buf[0..n].copy_from_slice(s.as_bytes());
        Ok(n)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, ()> {
        let s = String::from_utf8_lossy(buf);
        let n = s.len();
        print_fmt(format_args!("{}", s));
        Ok(n)
    }
}

pub fn cols() -> usize {
    if cfg!(feature = "video") {
        sys::vga::cols()
    } else {
        80
    }
}

pub fn rows() -> usize {
    if cfg!(feature = "video") {
        sys::vga::rows()
    } else {
        25
    }
}

pub fn has_cursor() -> bool {
    cfg!(feature = "video")
}

pub fn disable_echo() {
    let mut echo = ECHO.lock();
    *echo = false;
}

pub fn enable_echo() {
    let mut echo = ECHO.lock();
    *echo = true;
}

pub fn is_echo_enabled() -> bool {
    *ECHO.lock()
}

pub fn disable_raw() {
    let mut raw = RAW.lock();
    *raw = false;
}

pub fn enable_raw() {
    let mut raw = RAW.lock();
    *raw = true;
}

pub fn is_raw_enabled() -> bool {
    *RAW.lock()
}

const ETX_KEY: char = '\x03'; // End of Text
const EOT_KEY: char = '\x04'; // End of Transmission
const BS_KEY:  char = '\x08'; // Backspace
const ESC_KEY: char = '\x1B'; // Escape

pub fn key_handle(key: char) {
    let mut stdin = STDIN.lock();

    if key == BS_KEY && !is_raw_enabled() {
        // Avoid printing more backspaces than chars inserted into STDIN
        if let Some(c) = stdin.pop() {
            if is_echo_enabled() {
                let n = match c {
                    ETX_KEY | EOT_KEY | ESC_KEY => 2,
                    _ => c.len_utf8(),
                };
                print_fmt(format_args!("{}", BS_KEY.to_string().repeat(n)));
            }
        }
    } else {
        stdin.push(key);
        if is_echo_enabled() {
            match key {
                ETX_KEY => print_fmt(format_args!("^C")),
                EOT_KEY => print_fmt(format_args!("^D")),
                ESC_KEY => print_fmt(format_args!("^[")),
                _       => print_fmt(format_args!("{}", key)),
            };
        }
    }
}

pub fn end_of_text() -> bool {
    interrupts::without_interrupts(|| {
        STDIN.lock().contains(ETX_KEY)
    })
}

pub fn end_of_transmission() -> bool {
    interrupts::without_interrupts(|| {
        STDIN.lock().contains(EOT_KEY)
    })
}

pub fn drain() {
    interrupts::without_interrupts(|| {
        STDIN.lock().clear();
    })
}

pub fn read_char() -> char {
    sys::console::disable_echo();
    sys::console::enable_raw();
    loop {
        sys::time::halt();
        let res = interrupts::without_interrupts(|| {
            let mut stdin = STDIN.lock();
            if !stdin.is_empty() {
                Some(stdin.remove(0))
            } else {
                None
            }
        });
        if let Some(c) = res {
            sys::console::enable_echo();
            sys::console::disable_raw();
            return c;
        }
    }
}

pub fn read_line() -> String {
    loop {
        sys::time::halt();
        let res = interrupts::without_interrupts(|| {
            let mut stdin = STDIN.lock();
            match stdin.chars().next_back() {
                Some('\n') => {
                    let line = stdin.clone();
                    stdin.clear();
                    Some(line)
                }
                _ => {
                    None
                }
            }
        });
        if let Some(line) = res {
            return line;
        }
    }
}

#[doc(hidden)]
pub fn print_fmt(args: fmt::Arguments) {
    if cfg!(feature = "video") {
        sys::vga::print_fmt(args);
    } else {
        sys::serial::print_fmt(args);
    }
}
