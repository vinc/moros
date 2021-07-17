use crate::sys;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::port::Port;

// TODO: Support dyn KeyboardLayout

#[cfg(feature = "qwerty")]
lazy_static! {
    pub static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(
        layouts::Us104Key,
        ScancodeSet1,
        HandleControl::MapLettersToUnicode
    ));
}

#[cfg(feature = "dvorak")]
lazy_static! {
    pub static ref KEYBOARD: Mutex<Keyboard<layouts::Dvorak104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(
        layouts::Dvorak104Key,
        ScancodeSet1,
        HandleControl::MapLettersToUnicode
    ));
}

pub fn init() {
    /*
    let mut port = Port::new(0x60);

    // Identify
    let res = unsafe {
        port.write(0xF2 as u8); // Identify
        port.read()
    };
    if res != 0xFA { // 0xFA == ACK, 0xFE == Resend
        return init();
    }
    let res = unsafe {
        port.read()
    };
    printk!("[{:.6}] keyboard: identify {:#X}\n", sys::clock::uptime(), res);
    let res = unsafe {
        port.read()
    };
    printk!("[{:.6}] keyboard: identify {:#X}\n", sys::clock::uptime(), res);

    // Self-test
    let res = unsafe {
        port.write(0xFF as u8); // Reset and self-test
        port.read()
    };
    if res != 0xFA { // 0xFA == ACK, 0xFE == Resend
        return init();
    }
    let res = unsafe {
        port.read()
    };
    if res == 0xAA { // 0xAA == Passed, 0xFC or 0xFD == Failed, 0xFE == Resend
        printk!("[{:.6}] keyboard: self test passed\n", sys::clock::uptime());
    } else {
        printk!("[{:.6}] keyboard: self test failed ({:#X})\n", sys::clock::uptime(), res);
    }

    // Switch to scancode set 2
    // TODO: Not working because PS/2 controller is configured to do the translation (0xAB, 0x41)
    let res = unsafe {
        port.write(0xF0 as u8); // Set current scancode set
        port.write(0x02 as u8); // to 2
        port.read()
    };
    if res != 0xFA { // 0xFA == ACK, 0xFE == Resend
        return init();
    }
    printk!("[{:.6}] keyboard: switch to scancode set 2\n", sys::clock::uptime());
    */
    sys::idt::set_irq_handler(1, interrupt_handler);
}

fn read_scancode() -> u8 {
    let mut port = Port::new(0x60);
    unsafe { port.read() }
}

fn interrupt_handler() {
    let mut keyboard = KEYBOARD.lock();
    let scancode = read_scancode();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            let c = match key {
                DecodedKey::Unicode(c) => c,
                DecodedKey::RawKey(KeyCode::ArrowLeft)  => '←', // U+2190
                DecodedKey::RawKey(KeyCode::ArrowUp)    => '↑', // U+2191
                DecodedKey::RawKey(KeyCode::ArrowRight) => '→', // U+2192
                DecodedKey::RawKey(KeyCode::ArrowDown)  => '↓', // U+2193
                DecodedKey::RawKey(_) => { return; }
            };
            sys::console::key_handle(c);
        }
    }
}
