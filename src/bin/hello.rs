#![no_std]
#![no_main]

extern crate alloc;

use moros::entry_point;
use moros::api::syscall;
use moros::api::allocator::ALLOCATOR;

use alloc::string::String;

entry_point!(main);

fn main(args: &[&str]) -> usize {
    syscall::write(1, b"Hello, World!\n");
    let heap_start = args.as_ptr() as usize + (1 << 16);
    let heap_size = 1 << 16;
    ALLOCATOR.lock().init(heap_start, heap_size);

    let s = String::from("Allocated string");
    syscall::write(1, s.as_bytes());
    0
}
