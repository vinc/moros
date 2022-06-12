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
pub unsafe extern "sysv64" fn _start(args_ptr: u64, args_len: usize) {
    let args = core::slice::from_raw_parts(args_ptr as *const &str, args_len);
    let code = main(args);
    syscall::exit(code);
}

fn main(args: &[&str]) -> usize {
    syscall::write(1, b"Hello, World!\n");
    //syscall::stop(args.as_ptr() as usize); // DEBUG
    //syscall::stop(args.len()); // DEBUG
    let n = args.len();
    for i in 0..n {
        syscall::write(1, b"arg: ");
        //syscall::stop(args[i].as_ptr() as usize); // DEBUG
        //syscall::stop(args[i].len()); // DEBUG
        syscall::write(1, args[i].as_bytes());
        syscall::write(1, b"\n");
    }
    0
}
