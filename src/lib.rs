#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

#[cfg(target_arch = "x86_64")]
extern crate alloc;

#[macro_use]
#[cfg(target_arch = "x86_64")]
pub mod api;

#[macro_use]
pub mod sys;

#[cfg(target_arch = "x86_64")]
pub mod usr;

#[cfg(target_arch = "x86_64")]
use sys::boot::MemoryMap;

#[cfg(target_arch = "x86_64")]
pub const KERNEL_SIZE: usize = 4 << 20; // 4 MB

pub const STACK_SIZE: usize = 128 << 10; // 128 KB

#[cfg(target_arch = "x86_64")]
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

#[cfg(target_arch = "x86_64")]
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

#[allow(dead_code)]
#[cfg(target_arch = "x86_64")]
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

pub trait Testable {
    fn run(&self);
}

#[cfg(target_arch = "x86_64")]
impl<T> Testable for T where T: Fn() {
    fn run(&self) {
        printk!("test {} ... ", core::any::type_name::<T>());
        self();
        let csi_color = api::console::Style::color("lime");
        let csi_reset = api::console::Style::reset();
        printk!("{}ok{}\n", csi_color, csi_reset);
    }
}

#[cfg(target_arch = "x86_64")]
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

pub fn hang() -> ! {
    loop {
        sys::x86::hlt();
    }
}

#[cfg(test)]
use bootloader::{entry_point, BootInfo};

#[cfg(test)]
use core::panic::PanicInfo;

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    let memory_map = sys::boot::bootloader::extract_memory_map(boot_info);
    let offset = boot_info.physical_memory_offset;
    init(&memory_map, offset);
    test_main();
    hang();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let csi_color = api::console::Style::color("red");
    let csi_reset = api::console::Style::reset();
    printk!("{}failed{}\n\n", csi_color, csi_reset);
    printk!("{}\n\n", info);
    exit_qemu(QemuExitCode::Failed);
    hang();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
