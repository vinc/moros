#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moros::print;
use moros::kernel::sleep::sleep;
use moros::user::shell::Shell;

entry_point!(main);

fn main(_boot_info: &'static BootInfo) -> ! {
    moros::init();

    let mut shell = Shell::new();
    shell.run();

    loop { sleep(10.0) }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    loop { sleep(10.0) }
}
