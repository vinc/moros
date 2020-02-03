use crate::{print, kernel, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let (fg, bg) = kernel::vga::color();

    for &c in &kernel::vga::colors() {
        kernel::vga::set_color(c, bg);
        print!(" {:02} ", c as u8);
    }
    kernel::vga::set_color(fg, bg);
    print!("\n");

    for &c in &kernel::vga::colors() {
        kernel::vga::set_color(bg, c);
        print!(" {:02} ", c as u8);
    }
    kernel::vga::set_color(fg, bg);
    print!("\n");

    user::shell::ExitCode::CommandSuccessful
}

