use core::cmp;
use crate::{print, kernel, user};
use alloc::vec::Vec;
use alloc::string::String;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let pathname = args[1];
    let mut editor = Editor::new(pathname);
    editor.run()
}

pub struct Editor {
    file: Option<kernel::fs::File>,
    pathname: String,
    lines: Vec<String>,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        let mut lines = Vec::new();

        let file = match kernel::fs::File::open(pathname) {
            Some(file) => {
                let contents = file.read_to_string();
                for line in contents.split('\n') {
                    lines.push(line.into());
                }
                Some(file)
            },
            None => {
                lines.push(String::new());
                kernel::fs::File::create(pathname)
            }
        };

        let pathname = pathname.into();

        Self { file, pathname, lines }
    }

    pub fn save(&self) -> user::shell::ExitCode {
        if self.pathname.starts_with("/dev") || self.pathname.starts_with("/sys") {
            print!("Permission denied to write to '{}'\n", self.pathname);
            user::shell::ExitCode::CommandError
        } else if self.file.is_some() {
            let mut contents = String::new();
            let n = self.lines.len();
            for i in 0..n {
                contents.push_str(&self.lines[i]);
                if i < n - 1 {
                    contents.push('\n');
                }
            }
            self.file.unwrap().write(&contents.as_bytes()).unwrap();
            user::shell::ExitCode::CommandSuccessful
        } else {
            print!("Could not write to '{}'\n", self.pathname);
            user::shell::ExitCode::CommandError
        }
    }

    fn print_screen(&mut self) {
        kernel::vga::clear_screen();
        for line in &self.lines {
            print!("{}\n", line);
        }
    }

    pub fn run(&mut self) -> user::shell::ExitCode {
        self.print_screen();
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
                    // TODO: Allow more lines than screen height
                    if y < kernel::vga::screen_height() - 1 {
                        let new_line = self.lines[y].split_off(x);
                        self.lines.insert(y + 1, new_line);
                        self.print_screen();
                        kernel::vga::set_cursor_position(0, y + 1);
                        kernel::vga::set_writer_position(0, y + 1);
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
                '\x01' => { // Ctrl A
                    let x = 0;
                    kernel::vga::set_cursor_position(x, y);
                    kernel::vga::set_writer_position(x, y);
                },
                '\x05' => { // Ctrl E
                    let x = self.lines[y].len() - 1;
                    kernel::vga::set_cursor_position(x, y);
                    kernel::vga::set_writer_position(x, y);
                },
                '\x08' => { // Backspace
                    if x > 0 {
                        let line = self.lines[y].clone();
                        let (before_cursor, mut after_cursor) = line.split_at(x - 1);
                        if after_cursor.len() > 0 {
                            after_cursor = &after_cursor[1..];
                        }
                        self.lines[y].clear();
                        self.lines[y].push_str(before_cursor);
                        self.lines[y].push_str(after_cursor);
                        kernel::vga::clear_row();
                        print!("{}", self.lines[y]);
                        kernel::vga::set_cursor_position(x - 1, y);
                        kernel::vga::set_writer_position(x - 1, y);
                    } else {
                        if y > 0 {
                            let x = self.lines[y - 1].len();
                            let line = self.lines.remove(y);
                            self.lines[y - 1].push_str(&line);
                            self.print_screen();
                            kernel::vga::set_cursor_position(x, y - 1);
                            kernel::vga::set_writer_position(x, y - 1);
                        }
                    }
                },
                c => {
                    // TODO: Allow more chars than screen width
                    if self.lines[y].len() < kernel::vga::screen_width() {
                        if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                            let line = self.lines[y].clone();
                            let (before_cursor, after_cursor) = line.split_at(x);
                            self.lines[y].clear();
                            self.lines[y].push_str(before_cursor);
                            self.lines[y].push(c);
                            self.lines[y].push_str(after_cursor);
                            kernel::vga::clear_row();
                            print!("{}", self.lines[y]);
                            if x == kernel::vga::screen_width() - 1 {
                                kernel::vga::set_cursor_position(x, y);
                                kernel::vga::set_writer_position(x, y);
                            } else {
                                kernel::vga::set_cursor_position(x + 1, y);
                                kernel::vga::set_writer_position(x + 1, y);
                            }
                        }
                    }
                },
            }
        }
        user::shell::ExitCode::CommandSuccessful
    }
}
