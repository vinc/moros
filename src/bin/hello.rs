#![no_std]
#![no_main]

extern crate alloc;

use moros::entry_point;
use moros::api::syscall;

use alloc::string::String;

entry_point!(main);

fn main(args: &[&str]) -> usize {
    syscall::write(1, b"Hello, World!\n");
    let heap_start = args.as_ptr() as usize + (1 << 16);
    let heap_size = 1 << 16;
    if ALLOCATOR.is_locked() {
        syscall::write(1, b"Allocator is locked\n");
    } else {
        syscall::write(1, b"Allocator is not locked\n");
    }
    if let Some(mut alloc) = ALLOCATOR.try_lock() {
        syscall::write(1, b"Got a lock on the allocator\n");
        unsafe { alloc.init(heap_start, heap_size) };
        let s = String::from("Allocated string");
        syscall::write(1, s.as_bytes());
        0
    } else {
        syscall::write(1, b"Could not get a lock on the allocator\n");
        1
    }
}
