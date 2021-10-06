#![no_std]
#![no_main]
#![feature(asm)]
#![feature(naked_functions)]

use moros::api::syscall;
use core::panic::PanicInfo;

#[no_mangle]
#[naked]
pub unsafe extern "sysv64" fn _start() -> ! {
    asm!(
        "call {}",
        sym main,
        options(noreturn)
    )
}

fn main(_argc: isize, _argv: *mut *const u8) {
    loop {
        syscall::write(1, b"Hello, World!\n");
        syscall::sleep(1.0);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
