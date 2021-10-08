#![no_std]
#![no_main]

use moros::api::syscall;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "sysv64" fn _start() -> ! {
    loop {
        syscall::sleep(1.0);
    }
}
