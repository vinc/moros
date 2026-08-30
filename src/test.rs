use crate::api;
use crate::sys;

#[cfg(target_arch = "x86_64")]
use bootloader::{entry_point, BootInfo};

use core::panic::PanicInfo;

// MOROS use rust-bootloader for the tests on AMD64 and multiboot2 on i686

#[cfg(target_arch = "x86_64")]
entry_point!(test_kernel_main);

#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    // Multiboot2 does not provide a stack
    ".section .bss",
    ".align 16",
    "stack_bottom:",
    ".skip {size}",
    "stack_top:",

    ".section .text",
    ".global _start",
    "_start:",
    "mov esp, offset stack_top",
    "push eax", // magic
    "push ebx", // info
    "call {start}",
    "hlt",
    size = const crate::STACK_SIZE,
    start = sym test_kernel_main,
);

#[cfg(target_arch = "x86_64")]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    let memory_map = sys::boot::bootloader::extract_memory_map(boot_info);
    let offset = boot_info.physical_memory_offset;
    crate::init(&memory_map, offset);
    crate::test_main();
    crate::hang();
}

#[cfg(target_arch = "x86")]
extern "C" fn test_kernel_main(info: u32, magic: u32) -> ! {
    let memory_map = sys::boot::multiboot::extract_memory_map(info, magic);
    sys::boot::multiboot::init(&memory_map);
    //let offset = 0;
    //init(&memory_map, offset);
    crate::test_main();
    crate::hang();
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T where T: Fn() {
    fn run(&self) {
        printk!("test {} ... ", core::any::type_name::<T>());
        self();
        let csi_color = api::console::Style::color("lime");
        let csi_reset = api::console::Style::reset();
        printk!("{}ok{}\n", csi_color, csi_reset);
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    let n = tests.len();
    printk!("\nrunning {} test{}\n", n, if n == 1 { "" } else { "s" });
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
    unsafe {
        sys::x86::port::outl(0xF4, exit_code as u32);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let csi_color = api::console::Style::color("red");
    let csi_reset = api::console::Style::reset();
    printk!("{}failed{}\n\n", csi_color, csi_reset);
    printk!("{}\n\n", info);
    exit_qemu(QemuExitCode::Failed);
    crate::hang();
}
