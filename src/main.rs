#![no_std]
#![no_main]

extern crate rlibc; // TODO: See https://github.com/rust-lang/wg-cargo-std-aware/issues/53
extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moros::{kernel, print, user};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    moros::init(boot_info);
    loop {
        let bootrc = "/ini/boot.sh";
        if kernel::fs::File::open(bootrc).is_some() {
            user::shell::main(&["shell", bootrc]);
        } else {
            if kernel::fs::is_mounted() {
                print!("Could not find '{}'\n", bootrc);
            } else {
                print!("MFS is not mounted to '/'\n");
            }
            print!("Running console in diskless mode\n");

            user::shell::main(&["shell"]);
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    loop {
        kernel::time::sleep(10.0)
    }
}
