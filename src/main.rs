#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use alloc::string::ToString;
use moros::api::console::Style;
use moros::sys::mem::MemoryMap;
use moros::sys::mem::MemoryRegion;
use moros::sys::mem::MemoryRegionType;
use moros::{
    error, warning, hlt_loop, eprint, eprintln, print, println, sys, usr
};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use bootloader::bootinfo::MemoryRegionType as Mem;
    let mut memory_map = MemoryMap::new();
    for region in boot_info.memory_map.iter() {
        let addr = region.range.start_addr();
        let size = region.range.end_addr() - addr;
        let kind = match region.region_type {
            Mem::Usable => MemoryRegionType::Usable,
            _ => MemoryRegionType::Reserved,
        };
        memory_map.add(MemoryRegion::new(addr, size, kind));
    }
    let offset = boot_info.physical_memory_offset;
    moros::init(&memory_map, offset);
    print!("\x1b[?25h"); // Enable cursor
    loop {
        if let Some(cmd) = option_env!("MOROS_CMD") {
            let prompt = usr::shell::prompt_string(true);
            println!("{}{}", prompt, cmd);
            usr::shell::exec(cmd).ok();
            sys::acpi::shutdown();
        } else {
            user_boot();
        }
    }
}

fn user_boot() {
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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
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
    hlt_loop();
}
