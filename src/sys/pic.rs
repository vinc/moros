use crate::sys::x86::port::*;

// Ports
const PIC1_CMD: u16 = 0x20;
const PIC2_CMD: u16 = 0xA0;
const PIC1_DATA: u16 = PIC1_CMD + 1;
const PIC2_DATA: u16 = PIC2_CMD + 1;

// Vector offset
const PIC_OFFSET: u8 = 0x20;

// Initialization Command Words (ICW)
const ICW1_INIT: u8 = 0x10; // Init (A0=0, D4=1)
const ICW1_ICW4: u8 = 0x01; // ICW4 needed (D0=1)
const ICW4_8086: u8 = 0x01; // 8086 mode (D0=1)

// Operation Command Words (OCW)
const PIC_EOI: u8 = 0x20; // End-of-interrupt (D5=1)

// IRQ List
pub const PIT_IRQ: u8 = 0;
pub const KBD_IRQ: u8 = 1;
pub const PIC_IRQ: u8 = 2;
pub const COM_IRQ: u8 = 4;
pub const SND_IRQ: u8 = 5;
pub const RTC_IRQ: u8 = 8;

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
    PIC_OFFSET + irq
}

pub fn eoi(irq: u8) {
    unsafe {
        if irq >= 8 {
            outb(PIC2_CMD, PIC_EOI);
        }
        outb(PIC1_CMD, PIC_EOI);
    }
}

fn wait() {
    unsafe {
        outb(0x80, 0);
    }
}

fn mask_all() {
    unsafe {
        outb(PIC1_DATA, 0xFF);
        outb(PIC2_DATA, 0xFF);
    }
}

fn remap(offset: u8) {
    unsafe {
        // Init
        outb(PIC1_CMD, ICW1_INIT | ICW1_ICW4); // Clears the mask register
        wait();
        outb(PIC2_CMD, ICW1_INIT | ICW1_ICW4); // Clears the mask register
        wait();

        // Vector
        outb(PIC1_DATA, offset);
        wait();
        outb(PIC2_DATA, offset + 8);
        wait();

        // Cascade
        outb(PIC1_DATA, 1 << PIC_IRQ);
        wait();
        outb(PIC2_DATA, PIC_IRQ);
        wait();

        // Mode
        outb(PIC1_DATA, ICW4_8086);
        wait();
        outb(PIC2_DATA, ICW4_8086);
    }
}

pub fn init() {
    // Initially PIC1 has an offset of 0x8 which overlaps with processor
    // interrupts so we need to map them higher, but below the system call
    // interrupt at 0x80.
    remap(PIC_OFFSET);

    // The remapping unmasked everything
    mask_all();
    unmask(PIC_IRQ);
}
