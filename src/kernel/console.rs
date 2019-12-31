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

pub fn key_handle(key: DecodedKey) {
    let c = match key {
        DecodedKey::Unicode(c) => c,
        DecodedKey::RawKey(KeyCode::ArrowUp) => '↑',
        DecodedKey::RawKey(KeyCode::ArrowDown) => '↓',
        DecodedKey::RawKey(_) => '\0'
    };
    let mut stdin = STDIN.lock();
    stdin.push(c);
    if is_echo_enabled() {
        print!("{}", c);
    }
}

pub fn get_char() -> char {
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
