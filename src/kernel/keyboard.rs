use crate::kernel;
use lazy_static::lazy_static;
use pc_keyboard::{Keyboard, ScancodeSet1, HandleControl, layouts};
use spin::Mutex;
use x86_64::instructions::port::Port;

lazy_static! {
    // NOTE: Replace `Dvorak104Key` with `Us104Key` for Qwerty keyboards
    // TODO: Support layout change from userspace
    pub static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::MapLettersToUnicode)
    );
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
    print!("[{:.6}] keyboard: identify 0x{:X}\n", kernel::clock::clock_monotonic(), res);
    let res = unsafe {
        port.read()
    };
    print!("[{:.6}] keyboard: identify 0x{:X}\n", kernel::clock::clock_monotonic(), res);

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
        print!("[{:.6}] keyboard: self test passed\n", kernel::clock::clock_monotonic());
    } else {
        print!("[{:.6}] keyboard: self test failed (0x{:X})\n", kernel::clock::clock_monotonic(), res);
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
    print!("[{:.6}] keyboard: switch to scancode set 2\n", kernel::clock::clock_monotonic());
    */
    kernel::idt::set_irq_handler(1, interrupt_handler);
}

pub fn read_scancode() -> u8 {
    let mut port = Port::new(0x60);
    unsafe {
        port.read()
    }
}

fn interrupt_handler() {
    let mut keyboard = KEYBOARD.lock();
    let scancode = read_scancode();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            kernel::console::key_handle(key);
        }
    }
}
