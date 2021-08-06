#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moros::{sys, usr, print, println};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    moros::init(boot_info);

    let bin = include_bytes!("../dsk/bin/sleep");
    let process = sys::process::Process::create(bin);
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
