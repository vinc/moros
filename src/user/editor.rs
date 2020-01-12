use core::cmp;
use crate::{print, kernel, user};
use heapless::{String, Vec};
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
    lines: Vec<String<U256>, U256>,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        let pathname = pathname.into();
        let mut lines = Vec::new();
        lines.push(String::new());

        Self { pathname, lines }
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
                    print!("{}", c);
                    if y == self.lines.len() - 1 {
                        self.lines.push(String::new());
                    }
                },
                '↑' => { // Arrow up
                    if y > 0 {
                        let y = y - 1;
                        let x = cmp::min(x, self.lines[y].len());
                        kernel::vga::set_cursor_position(x, y);
                        kernel::vga::set_writer_position(x, y);
                    }
                },
                '↓' => { // Arrow down
                    if y < cmp::min(kernel::vga::screen_height() - 1, self.lines.len() - 1) {
                        let y = y + 1;
                        let x = cmp::min(x, self.lines[y].len());
                        kernel::vga::set_cursor_position(x, y);
                        kernel::vga::set_writer_position(x, y);
                    }
                },
                '←' => { // Arrow left
                    if x > 0 {
                        kernel::vga::set_cursor_position(x - 1, y);
                        kernel::vga::set_writer_position(x - 1, y);
                    }
                },
                '→' => { // Arrow right
                    if x < cmp::min(kernel::vga::screen_width() - 1, self.lines[y].len()) {
                        kernel::vga::set_cursor_position(x + 1, y);
                        kernel::vga::set_writer_position(x + 1, y);
                    }
                },
                '\x08' => { // Backspace
                    if x > 0 {
                        let line = self.lines[y].clone();
                        let (before_cursor, mut after_cursor) = line.split_at(x - 1);
                        if after_cursor.len() > 0 {
                            after_cursor = &after_cursor[1..];
                        }
                        self.lines[y].clear();
                        self.lines[y].push_str(before_cursor).unwrap();
                        self.lines[y].push_str(after_cursor).unwrap();
                        kernel::vga::clear_row();
                        print!("{}", self.lines[y]);
                        kernel::vga::set_cursor_position(x - 1, y);
                        kernel::vga::set_writer_position(x - 1, y);
                    } else {
                        if y > 0 {
                            // Remove last empty line
                            if y == self.lines.len() - 1 && self.lines[y].len() == 0 {
                                self.lines.pop();
                            }
                            kernel::vga::set_cursor_position(self.lines[y - 1].len(), y - 1);
                            kernel::vga::set_writer_position(self.lines[y - 1].len(), y - 1);
                        }
                    }
                },
                c => {
                    if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                        let line = self.lines[y].clone();
                        let (before_cursor, after_cursor) = line.split_at(x);
                        self.lines[y].clear();
                        self.lines[y].push_str(before_cursor).unwrap();
                        self.lines[y].push(c).unwrap();
                        self.lines[y].push_str(after_cursor).unwrap();
                        kernel::vga::clear_row();
                        print!("{}", self.lines[y]);
                        kernel::vga::set_cursor_position(x + 1, y);
                        kernel::vga::set_writer_position(x + 1, y);
                    }
                },
            }
        }
    }
}
