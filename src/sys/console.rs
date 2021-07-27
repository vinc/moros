use crate::sys;
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

pub fn clear_row_after(x: usize) {
    if cfg!(feature = "video") {
        sys::vga::clear_row_after(x);
    } else {
        sys::serial::print_fmt(format_args!("\r")); // Move cursor to begining of line
        sys::serial::print_fmt(format_args!("\x1b[{}C", x)); // Move cursor forward to position
        sys::serial::print_fmt(format_args!("\x1b[K")); // Clear line after position
    }
}

pub fn clear_row() {
    clear_row_after(0);
}

pub fn cursor_position() -> (usize, usize) {
    if cfg!(feature = "video") {
        sys::vga::cursor_position()
    } else {
        sys::serial::print_fmt(format_args!("\x1b[6n")); // Ask cursor position
        get_char(); // ESC
        get_char(); // [
        let mut x = String::new();
        let mut y = String::new();
        loop {
            let c = get_char();
            if c == ';' {
                break;
            } else {
                y.push(c);
            }
        }
        loop {
            let c = get_char();
            if c == 'R' {
                break;
            } else {
                x.push(c);
            }
        }
        (x.parse().unwrap_or(1), y.parse().unwrap_or(1))
    }
}

pub fn set_cursor_position(x: usize, y: usize) {
    if cfg!(feature = "video") {
        sys::vga::set_cursor_position(x, y);
    } else {
        sys::serial::print_fmt(format_args!("\x1b[{};{}H", y + 1, x + 1));
    }
}

pub fn set_writer_position(x: usize, y: usize) {
    if cfg!(feature = "video") {
        sys::vga::set_writer_position(x, y);
    } else {
        sys::serial::print_fmt(format_args!("\x1b[{};{}H", y + 1, x + 1));
    }
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

// TODO: Rename to `read_char()`
pub fn get_char() -> char {
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

// TODO: Rename to `read_line()`
pub fn get_line() -> String {
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
