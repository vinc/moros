#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

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

    log!("MOROS v{}\n", env!("CARGO_PKG_VERSION"));

    kernel::time::init();
    kernel::keyboard::init();
    kernel::serial::init();
    kernel::mem::init(boot_info);
    kernel::cpu::init();
    kernel::pci::init(); // Require MEM
    kernel::net::init(); // Require PCI
    kernel::ata::init();
    kernel::fs::init(); // Require ATA
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T where T: Fn() {
    fn run(&self) {
        print!("test {} ... ", core::any::type_name::<T>());
        self();
        let csi_color = kernel::console::Style::color("LightGreen");
        let csi_reset = kernel::console::Style::reset();
        print!("{}ok{}\n", csi_color, csi_reset);
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    let n = tests.len();
    print!("\nrunning {} test{}\n", n, if n == 1 { "" } else { "s" });
    for test in tests {
        test.run();
    }
}

#[cfg(test)]
use bootloader::entry_point;

#[cfg(test)]
use core::panic::PanicInfo;

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    kernel::acpi::shutdown();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    kernel::acpi::shutdown();
    loop {}
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
