#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use moros::api::{allocator, syscall};
use moros::entry_point;

entry_point!(main);

fn main(args: &[&str]) {
    if args.len() > 1 {
        syscall::write(1, args[1].as_bytes());    // FIXME: this is needed
        syscall::write(1, "\n".as_bytes());

        let mut hello = "Hello, ".to_string();
        hello.push_str(args[1]);                  // FIXME: for that to work
        hello.push_str("!\n");
        syscall::write(1, hello.as_bytes());
    } else {
        syscall::write(1, b"Hello, World!\n");
    }
}
