#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(asm_sym)]
#![feature(naked_functions)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[macro_use]
pub mod api;

#[macro_use]
pub mod sys;

pub mod usr;

use bootloader::BootInfo;

const KERNEL_SIZE: usize = 2 << 20; // 2 MB

pub fn init(boot_info: &'static BootInfo) {
    sys::vga::init();
    sys::gdt::init();
    sys::idt::init();
    sys::pic::init(); // Enable interrupts
    sys::serial::init();
    sys::keyboard::init();
    sys::time::init();

    log!("MOROS v{}\n", env!("CARGO_PKG_VERSION"));
    sys::mem::init(boot_info);
    sys::cpu::init();
    sys::pci::init(); // Require MEM
    sys::net::init(); // Require PCI
    sys::ata::init();
    sys::fs::init(); // Require ATA
    sys::clock::init(); // Require MEM
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    let csi_color = api::console::Style::color("LightRed");
    let csi_reset = api::console::Style::reset();
    printk!("{}Error:{} Could not allocate {} bytes\n", csi_color, csi_reset, layout.size());
    hlt_loop();
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T where T: Fn() {
    fn run(&self) {
        print!("test {} ... ", core::any::type_name::<T>());
        self();
        let csi_color = api::console::Style::color("LightGreen");
        let csi_reset = api::console::Style::reset();
        println!("{}ok{}", csi_color, csi_reset);
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    let n = tests.len();
    println!("\nrunning {} test{}", n, if n == 1 { "" } else { "s" });
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
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
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let csi_color = api::console::Style::color("LightRed");
    let csi_reset = api::console::Style::reset();
    println!("{}failed{}\n", csi_color, csi_reset);
    println!("{}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
