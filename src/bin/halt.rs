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
    syscall::write(1, b"\x1b[93m"); // Yellow
    syscall::write(1, b"MOROS has reached its fate, the system is now halting.\n");
    syscall::write(1, b"\x1b[0m"); // Reset
    syscall::sleep(0.5);
    syscall::halt();
    loop { syscall::sleep(1.0) }
}
