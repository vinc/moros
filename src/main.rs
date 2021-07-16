#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moros::{sys, usr, print};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    moros::init(boot_info);
    loop {
        let bootrc = "/ini/boot.sh";
        if sys::fs::File::open(bootrc).is_some() {
            usr::shell::main(&["shell", bootrc]);
        } else {
            if sys::fs::is_mounted() {
                print!("Could not find '{}'\n", bootrc);
            } else {
                print!("MFS is not mounted to '/'\n");
            }
            print!("Running console in diskless mode\n");

            usr::shell::main(&["shell"]);
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    loop {
        sys::time::sleep(10.0)
    }
}
