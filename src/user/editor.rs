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
    editor.run()
}

pub struct Editor {
    pathname: String<U256>,
    lines: Vec<String<U256>, U256>,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        let mut lines = Vec::new();

        if let Some(mut file) = kernel::fs::File::open(pathname) {
            let contents = file.read();
            for line in contents.split("\n") {
                lines.push(line.into());
            }
        } else {
            lines.push(String::new());
        }

        let pathname = pathname.into();

        Self { pathname, lines }
    }

    pub fn save(&mut self) -> user::shell::ExitCode {
        if self.pathname.starts_with("/dev") || self.pathname.starts_with("/sys") {
            print!("Permission denied to write to '{}'\n", self.pathname);
            user::shell::ExitCode::CommandError
        } else if let Some(mut file) = kernel::fs::File::create(&self.pathname) {
            let mut contents = String::<U2048>::new();
            let n = self.lines.len();
            for i in 0..n {
                contents.push_str(&self.lines[i]);
                if i < n - 1 {
                    contents.push('\n');
                }
            }
            file.write(&contents);
            user::shell::ExitCode::CommandSuccessful
        } else {
            print!("Could not write to '{}'\n", self.pathname);
            user::shell::ExitCode::CommandError
        }
    }

    pub fn run(&mut self) -> user::shell::ExitCode {
        kernel::vga::clear_screen();
        let n = self.lines.len();
        for i in 0..n {
            print!("{}", self.lines[i]);
            if i < n - 1 {
                print!("\n");
            }
        }
        kernel::vga::set_cursor_position(0, 0);
        kernel::vga::set_writer_position(0, 0);

        loop {
            let (x, y) = kernel::vga::cursor_position();
            let c = kernel::console::get_char();
            match c {
                '\0' => {
                    continue;
                }
                '\x11' => { // Ctrl Q
                    kernel::vga::clear_screen();
                    break;
                },
                '\x17' => { // Ctrl W
                    self.save();
                },
                '\x18' => { // Ctrl X
                    kernel::vga::clear_screen();
                    return self.save();
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
        user::shell::ExitCode::CommandSuccessful
    }
}
