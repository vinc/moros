use crate::{print, kernel};
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;

lazy_static! {
    pub static ref STDIN: Mutex<String> = Mutex::new(String::new());
    pub static ref ECHO: Mutex<bool> = Mutex::new(true);
    pub static ref RAW: Mutex<bool> = Mutex::new(false);
}

#[cfg(feature="vga")]
pub fn has_cursor() -> bool {
    true
}

#[cfg(feature="serial")]
pub fn has_cursor() -> bool {
    false
}

#[cfg(feature="vga")]
pub fn clear_row() {
    kernel::vga::clear_row();
}

#[cfg(feature="serial")]
pub fn clear_row() {
    print!("\x1b[2K\r");
}

#[cfg(feature="vga")]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::kernel::vga::print_fmt(format_args!($($arg)*));
    });
}

#[cfg(feature="serial")]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::kernel::serial::print_fmt(format_args!($($arg)*));
    });
}

#[cfg(feature="vga")]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        let uptime = $crate::kernel::clock::uptime();
        let (fg, bg) = $crate::kernel::vga::color();
        $crate::kernel::vga::set_color($crate::kernel::vga::Color::Green, bg);
        $crate::kernel::vga::print_fmt(format_args!("[{:.6}] ", uptime));
        $crate::kernel::vga::set_color(fg, bg);
        $crate::kernel::vga::print_fmt(format_args!($($arg)*));
    });
}

#[cfg(feature="serial")]
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        let uptime = $crate::kernel::clock::uptime();
        $crate::kernel::serial::print_fmt(format_args!("[{:.6}] ", uptime));
        $crate::kernel::serial::print_fmt(format_args!($($arg)*));
    });
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

pub fn key_handle(key: char) {
    let mut stdin = STDIN.lock();

    if key == '\x08' && !is_raw_enabled() {
        // Avoid printing more backspaces than chars inserted into STDIN.
        // Also, the VGA driver support only ASCII so unicode chars will
        // be displayed with one square for each codepoint.
        if stdin.len() > 0 {
            let n = stdin.pop().unwrap().len_utf8();
            if is_echo_enabled() {
                for _ in 0..n {
                    print!("\x08");
                }
            }
        }
    } else {
        // TODO: Replace non-ascii chars by ascii square symbol to keep length
        // at 1 instead of being variable?
        stdin.push(key);
        if is_echo_enabled() {
            print!("{}", key);
        }
    }
}

pub fn get_char() -> char {
    kernel::console::disable_echo();
    kernel::console::enable_raw();
    loop {
        kernel::time::halt();
        let res = interrupts::without_interrupts(|| {
            let mut stdin = STDIN.lock();
            match stdin.chars().next_back() {
                Some(c) => {
                    stdin.clear();
                    Some(c)
                },
                _ => {
                    None
                }
            }
        });
        if let Some(c) = res {
            kernel::console::enable_echo();
            kernel::console::disable_raw();
            return c;
        }
    }
}

pub fn get_line() -> String {
    loop {
        kernel::time::halt();
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
