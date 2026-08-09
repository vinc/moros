use crate::sys::x86::port::*;

use pic8259::ChainedPics;
use spin::Mutex;

const PIC1_CMD: u16 = 0x20;
const PIC2_CMD: u16 = 0xA0;
const PIC1_DATA: u16 = PIC1_CMD + 1;
const PIC2_DATA: u16 = PIC2_CMD + 1;
const PIC1_OFFSET: u8 = 32;
const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET)
});

fn irq_port(irq: u8) -> u16 {
    if irq < 8 { PIC1_DATA } else { PIC2_DATA }
}

fn irq_line(irq: u8) -> u8 {
    if irq < 8 { irq } else { irq - 8 }
}

pub fn mask(irq: u8) {
    let port = irq_port(irq);
    unsafe {
        let value = inb(port) | (1 << irq_line(irq));
        outb(port, value);
    }
}

pub fn unmask(irq: u8) {
    let port = irq_port(irq);
    unsafe {
        let value = inb(port) & !(1 << irq_line(irq));
        outb(port, value);
    }
}

pub fn vector(irq: u8) -> u8 {
    PIC1_OFFSET + irq
}

pub fn eoi(irq: u8) {
    let i = vector(irq);
    unsafe {
        PICS.lock().notify_end_of_interrupt(i)
    }
}

pub fn init() {
    unsafe {
        PICS.lock().initialize();
    }
}
