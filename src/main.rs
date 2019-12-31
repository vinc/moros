#![no_std]
#![no_main]

use moros::print;
use moros::user::shell::Shell;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    moros::init();

    let mut shell = Shell::new();
    shell.run();

    moros::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    moros::hlt_loop();
}
