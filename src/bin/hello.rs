#![no_std]
#![no_main]

extern crate alloc;

use moros::api::syscall;
use moros::api::allocator::ALLOCATOR;

use alloc::string::String;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    syscall::write(1, b"An exception occured!\n");
    loop {}
}

#[no_mangle]
pub unsafe extern "sysv64" fn _start(args_ptr: u64, _args_len: usize) {
    let heap_start = args_ptr as usize + (1 << 16);
    let heap_size = 1 << 16;
    ALLOCATOR.lock().init(heap_start, heap_size);

    syscall::write(1, b"Hello, World!\n");

    let s = String::from("Hello, World");
    syscall::write(1, s.as_bytes());

    syscall::exit(0);
}
