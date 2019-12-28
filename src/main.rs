#![no_std]
#![no_main]

use moros::print;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    print!("********************************************************************************\n");
    print!("*                           Welcome to MOROS v0.1.0                            *\n");
    print!("********************************************************************************\n");
    print!("\n");

    moros::init();

    print!("> ");
    moros::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    moros::hlt_loop();
}
