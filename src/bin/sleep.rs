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
    let args = core::slice::from_raw_parts(args_ptr as *const _, args_len);
    let code = main(args);
    syscall::exit(code);
}

fn main(args: &[&str]) -> usize {
    if args.len() == 2 {
        if let Ok(duration) = args[1].parse::<f64>() {
            syscall::sleep(duration);
            return 0
        }
    }
    1
}
