#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod kernel;
pub mod user;

use bootloader::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    kernel::vga::init();
    kernel::gdt::init();
    kernel::idt::init();
    unsafe { kernel::pic::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    log!("MOROS version {}\n", env!("CARGO_PKG_VERSION"));

    kernel::time::init();
    kernel::keyboard::init();
    kernel::mem::init(boot_info);
    kernel::cpu::init();
    kernel::pci::init(); // Require MEM
    kernel::rtl8139::init(); // Require PCI
    kernel::ata::init();
    kernel::fs::init(); // Require ATA
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
