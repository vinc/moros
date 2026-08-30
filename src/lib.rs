#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

#[macro_use]
pub mod api;

#[macro_use]
pub mod sys;

#[cfg(target_arch = "x86_64")] // TODO: Remove
pub mod usr;

#[cfg(test)]
mod test;

use sys::boot::MemoryMap;

#[cfg(target_arch = "x86_64")]
pub const KERNEL_SIZE: usize = 4 << 20; // 4 MB

pub const STACK_SIZE: usize = 128 << 10; // 128 KB

#[cfg(target_arch = "x86_64")] // TODO: Remove
pub fn init(memory_map: &MemoryMap, offset: u64) {
    sys::vga::init();
    sys::gdt::init();
    sys::idt::init();
    sys::pic::init();

    sys::x86::int::enable_interrupts();

    sys::serial::init();
    sys::keyboard::init();
    sys::clk::init();

    let v = option_env!("MOROS_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
    log!("SYS MOROS v{}", v);

    sys::mem::init(memory_map, offset);
    sys::cpu::init();
    sys::acpi::init(); // Require MEM
    sys::rng::init();
    sys::pci::init(); // Require MEM
    sys::snd::init();
    sys::net::init(); // Require PCI
    sys::ata::init();
    sys::fs::init(); // Require ATA
    sys::process::init();

    log!("RTC {}", sys::clk::date());
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
pub fn exec() -> ! {
    print!("\x1b[?25h"); // Enable cursor
    loop {
        if let Some(cmd) = option_env!("MOROS_CMD") {
            let prompt = usr::shell::prompt_string(true);
            println!("{}{}", prompt, cmd);
            usr::shell::exec(cmd).ok();
            sys::acpi::shutdown();
        } else {
            let script = "/ini/boot.sh";
            if sys::fs::File::open(script).is_some() {
                usr::shell::main(&["shell", script]).ok();
            } else {
                if sys::fs::is_mounted() {
                    error!("Could not find '{}'", script);
                } else {
                    warning!("MFS not found, run 'install' to setup the system");
                }
                usr::shell::main(&["shell"]).ok();
            }
        }
    }
}

pub fn hang() -> ! {
    loop {
        sys::x86::hlt();
    }
}

#[allow(dead_code)]
#[cfg_attr(not(feature = "userspace"), alloc_error_handler)]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    let csi_color = api::console::Style::color("red");
    let csi_reset = api::console::Style::reset();
    printk!(
        "{}Error:{} Could not allocate {} bytes\n",
        csi_color,
        csi_reset,
        layout.size()
    );
    hang();
}

#[test_case]
fn test_lib() {
    assert_eq!(1, 1); // Trivial assertion
}
