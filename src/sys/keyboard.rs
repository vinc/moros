use crate::sys;
use crate::api::syscall;

use core::sync::atomic::{AtomicBool, Ordering};
use pc_keyboard::{layouts, DecodedKey, Error, HandleControl, KeyState, KeyCode, KeyEvent, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::port::Port;

pub static KEYBOARD: Mutex<Option<KeyboardLayout>> = Mutex::new(None);

pub static ALT: AtomicBool = AtomicBool::new(false);
pub static CTRL: AtomicBool = AtomicBool::new(false);
pub static SHIFT: AtomicBool = AtomicBool::new(false);

pub enum KeyboardLayout {
    Azerty(Keyboard<layouts::Azerty, ScancodeSet1>),
    Dvorak(Keyboard<layouts::Dvorak104Key, ScancodeSet1>),
    Qwerty(Keyboard<layouts::Us104Key, ScancodeSet1>),
}

impl KeyboardLayout {
    fn add_byte(&mut self, scancode: u8) -> Result<Option<KeyEvent>, Error> {
        match self {
            KeyboardLayout::Azerty(keyboard) => keyboard.add_byte(scancode),
            KeyboardLayout::Dvorak(keyboard) => keyboard.add_byte(scancode),
            KeyboardLayout::Qwerty(keyboard) => keyboard.add_byte(scancode),
        }
    }

    fn process_keyevent(&mut self, event: KeyEvent) -> Option<DecodedKey> {
        match self {
            KeyboardLayout::Azerty(keyboard) => keyboard.process_keyevent(event),
            KeyboardLayout::Dvorak(keyboard) => keyboard.process_keyevent(event),
            KeyboardLayout::Qwerty(keyboard) => keyboard.process_keyevent(event),
        }
    }

    fn from(name: &str) -> Option<Self> {
        match name {
            "azerty" => Some(KeyboardLayout::Azerty(Keyboard::new(HandleControl::MapLettersToUnicode))),
            "dvorak" => Some(KeyboardLayout::Dvorak(Keyboard::new(HandleControl::MapLettersToUnicode))),
            "qwerty" => Some(KeyboardLayout::Qwerty(Keyboard::new(HandleControl::MapLettersToUnicode))),
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

fn send_csi(code: &str) {
    send_key('\x1B'); // ESC
    send_key('[');
    for c in code.chars() {
        send_key(c);
    }
}

fn interrupt_handler() {
    if let Some(ref mut keyboard) = *KEYBOARD.lock() {
        let scancode = read_scancode();
        if let Ok(Some(event)) = keyboard.add_byte(scancode) {
            let ord = Ordering::Relaxed;
            match event.code {
                KeyCode::AltLeft | KeyCode::AltRight => ALT.store(event.state == KeyState::Down, ord),
                KeyCode::ShiftLeft | KeyCode::ShiftRight => SHIFT.store(event.state == KeyState::Down, ord),
                KeyCode::ControlLeft | KeyCode::ControlRight => CTRL.store(event.state == KeyState::Down, ord),
                _ => {}
            }
            let is_alt = ALT.load(ord);
            let is_ctrl = CTRL.load(ord);
            let is_shift = SHIFT.load(ord);
            if let Some(key) = keyboard.process_keyevent(event) {
                match key {
                    DecodedKey::Unicode('\u{7f}') if is_alt && is_ctrl => syscall::reboot(), // Ctrl-Alt-Del
                    DecodedKey::RawKey(KeyCode::PageUp)     => send_csi("5~"),
                    DecodedKey::RawKey(KeyCode::PageDown)   => send_csi("6~"),
                    DecodedKey::RawKey(KeyCode::ArrowUp)    => send_csi("A"),
                    DecodedKey::RawKey(KeyCode::ArrowDown)  => send_csi("B"),
                    DecodedKey::RawKey(KeyCode::ArrowRight) => send_csi("C"),
                    DecodedKey::RawKey(KeyCode::ArrowLeft)  => send_csi("D"),
                    DecodedKey::Unicode('\t') if is_shift   => send_csi("Z"), // Convert Shift-Tab into Backtab
                    DecodedKey::Unicode(c)                  => send_key(c),
                    _ => {},
                };
            }
        }
    }
}
