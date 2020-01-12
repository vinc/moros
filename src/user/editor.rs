use crate::{print, kernel, user};
use heapless::String;
use heapless::consts::*;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let pathname = args[1];
    let mut editor = Editor::new(pathname);
    editor.run();
    user::shell::ExitCode::CommandSuccessful
}

fn input() -> String<U2048> {
    let mut output = String::new();
    loop {
        let line = kernel::console::get_line();
        if line == ".\n" {
            break;
        }
        output.push_str(&line).ok(); // TODO: File full
    }
    output
}

pub struct Editor {
    pathname: String<U256>,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        Self {
            pathname: pathname.into(),
        }
    }

    pub fn run(&mut self) {
        kernel::vga::clear_screen();
        loop {
            let (x, y) = kernel::vga::cursor_position();
            let c = kernel::console::get_char();
            match c {
                '\0' => {
                    continue;
                }
                '\x03' => { // Ctrl C
                    return;
                },
                '\n' => { // Newline
                    print!("\n");
                },
                '↑' => { // Arrow up
                    if y > 0 {
                        kernel::vga::set_cursor_position(x, y - 1);
                        kernel::vga::set_writer_position(x, y - 1);
                    }
                },
                '↓' => { // Arrow down
                    if y < kernel::vga::screen_height() - 1 {
                        kernel::vga::set_cursor_position(x, y + 1);
                        kernel::vga::set_writer_position(x, y + 1);
                    }
                },
                '←' => { // Arrow left
                    if x > 0 {
                        kernel::vga::set_cursor_position(x - 1, y);
                        kernel::vga::set_writer_position(x - 1, y);
                    }
                },
                '→' => { // Arrow right
                    if x < kernel::vga::screen_width() - 1 {
                        kernel::vga::set_cursor_position(x + 1, y);
                        kernel::vga::set_writer_position(x + 1, y);
                    }
                },
                '\x08' => { // Backspace
                    if x > 0 {
                        kernel::vga::set_cursor_position(x - 1, y);
                        kernel::vga::set_writer_position(x - 1, y);
                    }
                },
                c => {
                    if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                        print!("{}", c);
                    }
                },
            }
        }
    }
}
