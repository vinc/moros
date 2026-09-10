#![no_std]
#![no_main]

use moros::api::syscall;
use moros::entry_point;

entry_point!(main);

fn main(args: &[&str]) {
    syscall::write(1, args[1..].join(" ").as_bytes());
    syscall::write(1, b"\n");
}
