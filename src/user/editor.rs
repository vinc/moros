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
    offset: usize,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        let offset = 0;
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

        Self { file, pathname, lines, offset }
    }

    pub fn save(&self) -> user::shell::ExitCode {
        if self.file.is_some() {
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
        let from = self.offset;
        let to = cmp::min(self.lines.len(), self.offset + self.height());
        let lines = self.lines[from..to].join("\n");
        print!("{}", lines);
    }

    pub fn run(&mut self) -> user::shell::ExitCode {
        self.print_screen();
        kernel::vga::set_cursor_position(0, 0);
        kernel::vga::set_writer_position(0, 0);

        loop {
            let (mut x, mut y) = kernel::vga::cursor_position();
            let c = kernel::console::get_char();
            match c {
                '\0' => {
                    continue;
                }
                '\x03' => { // Ctrl C
                    kernel::vga::clear_screen();
                    break;
                }
                '\x11' => { // Ctrl Q
                    // TODO: Warn if modifications have not been saved
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
                    let new_line = self.lines[self.offset + y].split_off(x);
                    self.lines.insert(self.offset + y + 1, new_line);
                    if y == self.height() - 1 {
                        self.offset += 1;
                    } else {
                        y += 1;
                    }
                    x = 0;
                    self.print_screen();
                },
                '↑' => { // Arrow up
                    if y > 0 {
                        y -= 1
                    } else {
                        if self.offset > 0 {
                            self.offset -= 1;
                            self.print_screen();
                        }
                    }
                    x = cmp::min(x, self.lines[self.offset + y].len());
                },
                '↓' => { // Arrow down
                    let is_bottom = y == self.height() - 1;
                    let is_eof = self.offset + y == self.lines.len() - 1;
                    if y < cmp::min(self.height(), self.lines.len() - 1) {
                        if is_bottom || is_eof {
                            if !is_eof {
                                self.offset += 1;
                                self.print_screen();
                            }
                        } else {
                            y += 1;
                        }
                        x = cmp::min(x, self.lines[self.offset + y].len());
                    }
                },
                '←' => { // Arrow left
                    if x == 0 {
                        continue;
                    }
                    x -= 1;
                },
                '→' => { // Arrow right
                    let line = &self.lines[self.offset + y];
                    if x == cmp::min(self.width() - 1, line.len()) {
                        continue;
                    }
                    x += 1;
                },
                '\x14' => { // Ctrl T
                    x = 0;
                    y = 0;
                    self.offset = 0;
                    self.print_screen();
                },
                '\x02' => { // Ctrl B
                    x = 0;
                    y = cmp::min(self.height(), self.lines.len()) - 1;
                    self.offset = self.lines.len() - 1 - y;
                    self.print_screen();
                },
                '\x01' => { // Ctrl A
                    x = 0;
                },
                '\x05' => { // Ctrl E
                    x = self.lines[self.offset + y].len() - 1;
                },
                '\x08' => { // Backspace
                    if x > 0 { // Remove char from line
                        let line = self.lines[self.offset + y].clone();
                        let (before, mut after) = line.split_at(x - 1);
                        if after.len() > 0 {
                            after = &after[1..];
                        }
                        self.lines[self.offset + y].clear();
                        self.lines[self.offset + y].push_str(before);
                        self.lines[self.offset + y].push_str(after);
                        kernel::vga::clear_row();
                        print!("{}", self.lines[self.offset + y]);
                        x -= 1;
                    } else { // Remove newline char from previous line
                        if y == 0 && self.offset == 0 {
                            continue;
                        }

                        x = self.lines[self.offset + y - 1].len();
                        let line = self.lines.remove(self.offset + y);
                        self.lines[self.offset + y - 1].push_str(&line);
                        if y > 0 {
                            y -= 1;
                        } else {
                            self.offset -= 1;
                        }
                        self.print_screen();
                    }
                },
                c => {
                    let line = self.lines[self.offset + y].clone();

                    // TODO: Allow more chars than screen width
                    if line.len() == self.width() {
                        continue;
                    }
                    if !c.is_ascii_graphic() && !c.is_ascii_whitespace() {
                        continue;
                    }

                    let (before_cursor, after_cursor) = line.split_at(x);
                    self.lines[self.offset + y].clear();
                    self.lines[self.offset + y].push_str(before_cursor);
                    self.lines[self.offset + y].push(c);
                    self.lines[self.offset + y].push_str(after_cursor);
                    kernel::vga::clear_row();
                    print!("{}", self.lines[self.offset + y]);
                    if x < self.width() - 1 {
                        x += 1;
                    }
                },
            }
            kernel::vga::set_cursor_position(x, y);
            kernel::vga::set_writer_position(x, y);
        }
        user::shell::ExitCode::CommandSuccessful
    }

    fn height(&self) -> usize {
        kernel::vga::screen_height()
    }

    fn width(&self) -> usize {
        kernel::vga::screen_width()
    }
}
