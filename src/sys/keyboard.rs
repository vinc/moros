use crate::sys;

use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, Error, HandleControl, KeyCode, KeyEvent, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::port::Port;

lazy_static! {
    pub static ref KEYBOARD: Mutex<Option<KeyboardLayout>> = Mutex::new(None);
}

pub enum KeyboardLayout {
    Qwerty(Keyboard<layouts::Us104Key, ScancodeSet1>),
    Dvorak(Keyboard<layouts::Dvorak104Key, ScancodeSet1>),
}

impl KeyboardLayout {
    fn add_byte(&mut self, scancode: u8) -> Result<Option<KeyEvent>, Error> {
        match self {
            KeyboardLayout::Qwerty(keyboard) => keyboard.add_byte(scancode),
            KeyboardLayout::Dvorak(keyboard) => keyboard.add_byte(scancode),
        }
    }

    fn process_keyevent(&mut self, key_event: KeyEvent) -> Option<DecodedKey> {
        match self {
            KeyboardLayout::Qwerty(keyboard) => keyboard.process_keyevent(key_event),
            KeyboardLayout::Dvorak(keyboard) => keyboard.process_keyevent(key_event),
        }
    }

    fn from(name: &str) -> Option<Self> {
        match name {
            "qwerty" => Some(KeyboardLayout::Qwerty(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::MapLettersToUnicode))),
            "dvorak" => Some(KeyboardLayout::Dvorak(Keyboard::new(layouts::Dvorak104Key, ScancodeSet1, HandleControl::MapLettersToUnicode))),
            _ => None,
        }
    }
}

pub fn set_keyboard(layout: &str) -> bool {
    if let Some(keyboard) = KeyboardLayout::from(layout) {
        *KEYBOARD.lock() = Some(keyboard);
        true
    } else {
        false
    }
}

pub fn init() {
    set_keyboard(option_env!("MOROS_KEYBOARD").unwrap_or("qwerty"));

    sys::idt::set_irq_handler(1, interrupt_handler);
}

fn read_scancode() -> u8 {
    let mut port = Port::new(0x60);
    unsafe { port.read() }
}

fn send_key(c: char) {
    sys::console::key_handle(c);
}

fn send_csi(c: char) {
    send_key('\x1B'); // ESC
    send_key('[');
    send_key(c);
}

fn interrupt_handler() {
    if let Some(ref mut keyboard) = *KEYBOARD.lock() {
        let scancode = read_scancode();
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(c)                  => send_key(c),
                    DecodedKey::RawKey(KeyCode::ArrowUp)    => send_csi('A'),
                    DecodedKey::RawKey(KeyCode::ArrowDown)  => send_csi('B'),
                    DecodedKey::RawKey(KeyCode::ArrowRight) => send_csi('C'),
                    DecodedKey::RawKey(KeyCode::ArrowLeft)  => send_csi('D'),
                    _ => {},
                };
            }
        }
    }
}
