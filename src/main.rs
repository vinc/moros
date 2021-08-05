#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moros::{sys, usr, print, println};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    moros::init(boot_info);

    use x86_64::VirtAddr;
    let mut mapper = unsafe { sys::mem::mapper(VirtAddr::new(boot_info.physical_memory_offset)) };
    let mut frame_allocator = unsafe { sys::mem::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    let process = sys::process::Process::create(
        &mut mapper,
        &mut frame_allocator,
        &[
            // Infinite spin
            // 0x90, 0x90, 0x90, 0xEB, 0xFB

            // Infinite sleep (1.0 second at a time)
            0x48, 0xC7, 0xC0, 0x00, 0x00, 0x00, 0x00,                   // mov rax, 0x0
            0x48, 0xBF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // move rdi, 0x3ff0000000000000
            0x48, 0xC7, 0xC6, 0x00, 0x00, 0x00, 0x00,                   // mov rsi, 0x0
            0x48, 0xC7, 0xC2, 0x00, 0x00, 0x00, 0x00,                   // mov rdx, 0x0
            0xCD, 0x80,                                                 // int 0x80
            0xEB, 0xDD,                                                 // jmp 0
        ]
    );
    process.switch();

    loop {
        let bootrc = "/ini/boot.sh";
        if sys::fs::File::open(bootrc).is_some() {
            usr::shell::main(&["shell", bootrc]);
        } else {
            if sys::fs::is_mounted() {
                println!("Could not find '{}'", bootrc);
            } else {
                println!("MFS is not mounted to '/'");
            }
            println!("Running console in diskless mode");

            usr::shell::main(&["shell"]);
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        sys::time::sleep(10.0)
    }
}
