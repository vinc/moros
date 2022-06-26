#![no_std]
#![no_main]

use moros::api::syscall;
use moros::entry_point;

entry_point!(main);

fn main(_args: &[&str]) -> isize {
    syscall::write(1, b"Hello, World!\n");
    0
}
