#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;

use moros::api::syscall;
use moros::entry_point;

entry_point!(main);

fn main(args: &[&str]) {
    if args.len() > 1 {
        // FIXME: This will result in a page fault exception for an address
        // that's already mapped to the kernel stack
        syscall::write(1, format!("Hello, {}!\n", args[1]).as_bytes());
    } else {
        syscall::write(1, b"Hello, World!\n");
    }
}
