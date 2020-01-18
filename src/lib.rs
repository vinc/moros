#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

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
    kernel::ata::init();
    kernel::fs::init();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
