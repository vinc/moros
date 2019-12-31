use lazy_static::lazy_static;
use pc_keyboard::{KeyCode, DecodedKey};
use spin::Mutex;
use heapless::String;
use heapless::consts::*;
use crate::kernel::sleep::halt;
use x86_64::instructions::interrupts;

lazy_static! {
    pub static ref STDIN: Mutex<String<U256>> = Mutex::new(String::new());
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
}

pub fn get_char() -> char {
    loop {
        halt();
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

}
