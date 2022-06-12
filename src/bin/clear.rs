#![no_std]
#![no_main]

use moros::api::syscall;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    syscall::write(1, b"An exception occured!\n");
    loop {}
}

#[no_mangle]
pub unsafe extern "sysv64" fn _start() {
    syscall::write(1, b"\x1b[2J\x1b[1;1H"); // Clear screen and move cursor to top
    syscall::exit(0);
}
