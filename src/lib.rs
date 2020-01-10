#![no_std]
#![feature(abi_x86_interrupt)]

pub mod kernel;
pub mod user;

pub fn init() {
    //kernel::keyboard::init();
    kernel::gdt::init();
    kernel::interrupts::init_idt();
    unsafe { kernel::interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    kernel::cpu::init();
    kernel::pci::init();
    kernel::ata_pio::init();
}
