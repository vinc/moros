#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod kernel;
pub mod user;

use bootloader::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    kernel::gdt::init();
    kernel::idt::init();
    unsafe { kernel::pic::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    print!("[{:.6}] MOROS version {}\n", kernel::clock::clock_monotonic(), env!("CARGO_PKG_VERSION"));

    kernel::time::init();
    kernel::keyboard::init();
    kernel::mem::init(boot_info);
    kernel::cpu::init();
    kernel::pci::init();
    kernel::ata::init();
    kernel::fs::init();
    kernel::rtl8139::init();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
