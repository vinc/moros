#![no_std]
#![no_main]

use moros::print;
use moros::user::shell;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    moros::init();
    shell::print_banner();
    shell::print_prompt();
    moros::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    moros::hlt_loop();
}
