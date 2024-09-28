#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString;
use moros::api::syscall;
use moros::entry_point;

entry_point!(main);

fn main(args: &[&str]) {
    if args.len() > 1 {
        let n = args.len();
        for i in 1..n {
            let mut hello = "Hello, ".to_string();
            hello.push_str(args[i]);
            hello.push_str("!\n");
            syscall::write(1, hello.as_bytes());
        }
    } else {
        syscall::write(1, b"Hello, World!\n");
    }
}
