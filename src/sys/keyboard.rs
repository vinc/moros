use crate::api;
use crate::api::fs::{FileIO, IO};
use crate::sys;

use alloc::collections::vec_deque::VecDeque;
use alloc::format;
use core::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;
use pc_keyboard::{
    layouts, DecodedKey, Error, HandleControl, KeyCode, KeyEvent, KeyState,
    Keyboard, ScancodeSet1,
};
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

lazy_static! {
    pub static ref BUF: Mutex<VecDeque<u8>> = Mutex::new(VecDeque::new());
}

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
            KeyboardLayout::Azerty(kb) => kb.add_byte(scancode),
            KeyboardLayout::Dvorak(kb) => kb.add_byte(scancode),
            KeyboardLayout::Qwerty(kb) => kb.add_byte(scancode),
        }
    }

    fn process_keyevent(&mut self, event: KeyEvent) -> Option<DecodedKey> {
        match self {
            KeyboardLayout::Azerty(kb) => kb.process_keyevent(event),
            KeyboardLayout::Dvorak(kb) => kb.process_keyevent(event),
            KeyboardLayout::Qwerty(kb) => kb.process_keyevent(event),
        }
    }

    fn from(name: &str) -> Option<Self> {
        match name {
            "azerty" => Some(KeyboardLayout::Azerty(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Azerty,
                HandleControl::MapLettersToUnicode,
            ))),
            "dvorak" => Some(KeyboardLayout::Dvorak(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Dvorak104Key,
                HandleControl::MapLettersToUnicode,
            ))),
            "qwerty" => Some(KeyboardLayout::Qwerty(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::MapLettersToUnicode,
            ))),
            _ => None,
        }
    }
}

pub fn set_keyboard(layout: &str) -> bool {
    if let Some(keyboard) = KeyboardLayout::from(layout) {
        interrupts::without_interrupts(||
            *KEYBOARD.lock() = Some(keyboard)
        );
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

#[derive(Debug, Clone)]
pub struct KeyboardBuffer;

// Read scancodes from /dev/kbd/buffer
impl KeyboardBuffer {
    pub fn new() -> Self {
        interrupts::without_interrupts(|| BUF.lock().clear());
        Self {}
    }

    pub fn size() -> usize {
        4
    }
}

impl FileIO for KeyboardBuffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        interrupts::without_interrupts(||
            if let Some(scancode) = BUF.lock().pop_front() {
                buf[0] = scancode;
                Ok(1)
            } else {
                Ok(0)
            }
        )
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, ()> {
        Err(())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        interrupts::without_interrupts(||
            match event {
                IO::Read => !BUF.lock().is_empty(),
                IO::Write => false,
            }
        )
    }
}

fn interrupt_handler() {
    if let Some(ref mut keyboard) = *KEYBOARD.lock() {
        let scancode = read_scancode();

        let mut kb = BUF.lock();
        if kb.len() > 256 {
            kb.pop_front();
        }
        kb.push_back(scancode);

        if let Ok(Some(event)) = keyboard.add_byte(scancode) {
            let ord = Ordering::Relaxed;
            match event.code {
                KeyCode::LAlt | KeyCode::RAltGr => {
                    ALT.store(event.state == KeyState::Down, ord)
                }
                KeyCode::LShift | KeyCode::RShift => {
                    SHIFT.store(event.state == KeyState::Down, ord)
                }
                KeyCode::LControl | KeyCode::RControl => {
                    CTRL.store(event.state == KeyState::Down, ord)
                }
                _ => {}
            }
            let is_alt = ALT.load(ord);
            let is_ctrl = CTRL.load(ord);
            let is_shift = SHIFT.load(ord);
            if let Some(key) = keyboard.process_keyevent(event) {
                match key {
                    // Ctrl + Alt + Del
                    DecodedKey::Unicode('\u{7f}') if is_alt && is_ctrl => {
                        api::power::reboot()
                    }

                    // [Ctrl +] [Shift +] Tab
                    DecodedKey::Unicode('\t') => {
                        if is_ctrl {
                            if is_shift {
                                send_csi("1;6I")
                            } else {
                                send_csi("1;5I")
                            }
                        } else {
                            if is_shift {
                                send_csi("Z") // Backtab
                            } else {
                                send_key('\t') // Tab
                            }
                        }
                    }

                    DecodedKey::RawKey(KeyCode::ArrowUp) => {
                        if is_ctrl {
                            send_csi("1;5A")
                        } else if is_alt {
                            send_csi("1;3A")
                        } else {
                            send_csi("A")
                        }
                    }
                    DecodedKey::RawKey(KeyCode::ArrowDown) => {
                        if is_ctrl {
                            send_csi("1;5B")
                        } else if is_alt {
                            send_csi("1;3B")
                        } else {
                            send_csi("B")
                        }
                    }
                    DecodedKey::RawKey(KeyCode::ArrowRight) => {
                        if is_ctrl {
                            send_csi("1;5C")
                        } else if is_alt {
                            send_csi("1;3C")
                        } else {
                            send_csi("C")
                        }
                    }
                    DecodedKey::RawKey(KeyCode::ArrowLeft) => {
                        if is_ctrl {
                            send_csi("1;5D")
                        } else if is_alt {
                            send_csi("1;3D")
                        } else {
                            send_csi("D")
                        }
                    }

                    DecodedKey::RawKey(KeyCode::PageUp) => send_csi("5~"),
                    DecodedKey::RawKey(KeyCode::PageDown) => send_csi("6~"),

                    DecodedKey::Unicode(c) => {
                        let letter = (c as u8 | 0x40) as char;
                        if is_ctrl && is_shift && letter.is_ascii_uppercase() {
                            // Ctrl + Shift + Letter
                            send_csi(&format!("1;6{}", letter));
                        } else {
                            send_key(c)
                        }
                    }

                    _ => {}
                };
            }
        }
    }
}
