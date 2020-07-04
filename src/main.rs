#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moros::{print, user, kernel};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    moros::init(boot_info);
    loop {
        let bootrc = "/ini/boot.sh";
        if kernel::fs::File::open(bootrc).is_some() {
            user::shell::main(&["shell", bootrc]);
        } else {
            print!("Could not find '{}'\n", bootrc);
            print!("Running console in diskless mode\n");
            //print!("Use `mkfs` and `install` to setup MOROS on disk\n");
            user::shell::main(&["shell"]);
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    loop { kernel::time::sleep(10.0) }
}
