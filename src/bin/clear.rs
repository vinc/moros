#![no_std]
#![no_main]

use moros::api::syscall;
use moros::entry_point;

entry_point!(main);

fn main(_args: &[&str]) {
    syscall::write(1, b"\x1b[2J\x1b[1;1H"); // Clear screen and move cursor to top
}
