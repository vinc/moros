#![no_std]
#![no_main]

extern crate alloc;

use moros::print;
use moros::api::io;
use moros::api::syscall;
use moros::api::vga;
use moros::entry_point;

entry_point!(main);

fn main(_args: &[&str]) {
    vga::set_resolution("320x200p");
    print!("\x1b]R\x1b[1A"); // Reset palette
    while io::stdin().read_char().is_none() {
        syscall::sleep(0.1);
    }
    vga::set_resolution("80x25c");
}
