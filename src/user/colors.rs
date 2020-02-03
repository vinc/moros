use crate::{print, kernel, user};

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let colors = kernel::vga::colors();
    let (fg, bg) = kernel::vga::color();

    for i in 0..colors.len() {
        let c = colors[i];
        kernel::vga::set_color(c, bg);
        print!(" {:02} ", i);
        if i == 7 || i == 15 {
            kernel::vga::set_color(fg, bg);
            print!("\n");
        }
    }

    for i in 0..colors.len() {
        let c = colors[i];
        kernel::vga::set_color(bg, c);
        print!(" {:02} ", i);
        if i == 7 || i == 15 {
            kernel::vga::set_color(fg, bg);
            print!("\n");
        }
    }

    user::shell::ExitCode::CommandSuccessful
}

