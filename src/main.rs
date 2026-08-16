#![no_std]
#![no_main]

#[cfg(target_arch = "x86_64")]
extern crate alloc;

use core::panic::PanicInfo;

// MOROS supports 3 boot protocols: rust-bootloader, limine, and multiboot2

#[cfg(not(any(feature = "limine", feature = "multiboot")))]
#[no_mangle]
extern "C" fn _start(boot_info: &'static bootloader::BootInfo) -> ! {
    moros::sys::boot::bootloader::start(boot_info)
}

#[cfg(feature = "limine")]
#[no_mangle]
extern "C" fn _start() -> ! {
    moros::sys::boot::limine::start()
}

#[cfg(feature = "multiboot")]
core::arch::global_asm!(
    // Multiboot2 does not provide a stack
    ".section .bss",
    ".align 16",
    "stack_bottom:",
    ".skip 16384",
    "stack_top:",

    ".section .text",
    ".global _start",
    "_start:",
    "mov esp, offset stack_top",
    "push eax", // magic
    "push ebx", // info
    "call {start}",
    "hlt",
    start = sym start,
);

#[cfg(feature = "multiboot")]
extern "C" fn start(info: u32, magic: u32) -> ! {
    moros::sys::boot::multiboot::start(info, magic)
}

#[cfg(target_arch = "x86_64")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use alloc::string::ToString;
    use moros::api::console::Style;
    use moros::{error, hang, eprint, eprintln};
    if let Some(location) = info.location() {
        let title = "Panicked";
        let path = location.file();
        let row = location.line();
        let col = location.column();
        error!("{title} at {path}:{row}:{col}");

        let msg = info.message().to_string();
        if !msg.is_empty() {
            let red = Style::color("red");
            let reset = Style::reset();
            let space = " ".repeat("Error: ".len());
            let arrow = "^".repeat(title.len());
            eprintln!("{space}{red}{arrow} {msg}{reset}");
        }
    } else {
        error!("{info}");
    }
    hang();
}

#[cfg(target_arch = "x86")]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    moros::hang();
}
