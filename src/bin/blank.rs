#![no_std]
#![no_main]

extern crate alloc;

use moros::print;
use moros::api::io;
use moros::api::vga;
use moros::entry_point;

entry_point!(main);

fn main(_args: &[&str]) {
    vga::graphic_mode();
    print!("\x1b]R\x1b[1A"); // Reset palette
    while io::stdin().read_char().is_none() {
        x86_64::instructions::hlt();
    }
    vga::text_mode();
}
