use lazy_static::lazy_static;
use pc_keyboard::{KeyCode, DecodedKey};
use spin::RwLock;
use heapless::String;
use heapless::consts::*;
use x86_64::instructions::{interrupts, hlt};

lazy_static! {
    pub static ref STDIN: RwLock<String<U256>> = RwLock::new(String::new());
}

pub fn key_handle(key: DecodedKey) {
    let c = match key {
        DecodedKey::Unicode(c) => c,
        DecodedKey::RawKey(KeyCode::ArrowUp) => '↑',
        DecodedKey::RawKey(KeyCode::ArrowDown) => '↓',
        DecodedKey::RawKey(_) => '\0'
    };
    let mut stdin = STDIN.write();
    stdin.push(c);
}

// TODO: Add timeout
pub fn get_char() -> Option<char> {
    let mut c = None;

    while c.is_none() {
        hlt();
        interrupts::without_interrupts(|| {
            let stdin = STDIN.read();
            c = stdin.chars().next_back();
        });
    }

    interrupts::without_interrupts(|| {
        let mut stdin = STDIN.write();
        stdin.clear();
    });

    c
}
