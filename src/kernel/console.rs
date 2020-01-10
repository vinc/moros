use crate::{print, kernel};
use heapless::String;
use heapless::consts::*;
use lazy_static::lazy_static;
use pc_keyboard::{KeyCode, DecodedKey};
use spin::Mutex;
use x86_64::instructions::interrupts;

lazy_static! {
    pub static ref STDIN: Mutex<String<U256>> = Mutex::new(String::new());
    pub static ref ECHO: Mutex<bool> = Mutex::new(true);
    pub static ref RAW: Mutex<bool> = Mutex::new(false);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kernel::vga::print_fmt(format_args!($($arg)*)));
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

pub fn key_handle(key: DecodedKey) {
    let c = match key {
        DecodedKey::Unicode(c) => c,
        DecodedKey::RawKey(KeyCode::ArrowLeft)  => '←', // U+2190
        DecodedKey::RawKey(KeyCode::ArrowUp)    => '↑', // U+2191
        DecodedKey::RawKey(KeyCode::ArrowRight) => '→', // U+2192
        DecodedKey::RawKey(KeyCode::ArrowDown)  => '↓', // U+2193
        DecodedKey::RawKey(_) => '\0'
    };
    let mut stdin = STDIN.lock();

    if c == '\x08' && !is_raw_enabled() {
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
        if stdin.push(c).is_ok() && is_echo_enabled() {
            print!("{}", c);
        }
    }
}

pub fn get_char() -> char {
    kernel::console::disable_echo();
    kernel::console::enable_raw();
    loop {
        kernel::sleep::halt();
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

pub fn get_line() -> String<U256> {
    loop {
        kernel::sleep::halt();
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
