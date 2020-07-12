use crate::{kernel, print, user};
use crate::kernel::console::Style;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp;

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
    offset: usize, // TODO: Call it `offset_y` and introduce `offset_x`
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

    pub fn save(&mut self) -> user::shell::ExitCode {
        let mut contents = String::new();
        let n = self.lines.len();
        for i in 0..n {
            contents.push_str(&self.lines[i]);
            if i < n - 1 {
                contents.push('\n');
            }
        }

        let csi_reset = Style::reset();
        if let Some(file) = &mut self.file {
            file.write(&contents.as_bytes()).unwrap();
            let csi_color = Style::color("Yellow");
            self.print_status(&format!("{}Wrote {}L to '{}'{}", csi_color, n, self.pathname, csi_reset));
            user::shell::ExitCode::CommandSuccessful
        } else {
            let csi_color = Style::color("LightRed");
            self.print_status(&format!("{}Could not write to '{}'{}", csi_color, self.pathname, csi_reset));
            user::shell::ExitCode::CommandError
        }
    }

    fn print_status(&mut self, status: &str) {
        let (x, y) = kernel::vga::cursor_position();
        kernel::vga::set_writer_position(0, self.height());
        kernel::vga::set_cursor_position(0, self.height());
        print!("{}", status);
        kernel::vga::set_writer_position(x, y);
        kernel::vga::set_cursor_position(x, y);
    }

    fn print_screen(&mut self) {
        let mut lines: Vec<String> = Vec::new();
        let from = self.offset;
        let to = cmp::min(self.lines.len(), self.offset + self.height());
        for i in from..to {
            let n = cmp::min(self.lines[i].len(), self.width());
            lines.push(self.lines[i][0..n].into()) // TODO: Use `offset_x .. offset_x + n`
        }
        kernel::vga::clear_screen();
        print!("{}", lines.join("\n"));
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
                    let res = self.save();
                    kernel::vga::clear_screen();
                    return res;
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
                '\x14' => { // Ctrl T -> Go to top of file
                    x = 0;
                    y = 0;
                    self.offset = 0;
                    self.print_screen();
                },
                '\x02' => { // Ctrl B -> Go to bottom of file
                    x = 0;
                    y = cmp::min(self.height(), self.lines.len()) - 1;
                    self.offset = self.lines.len() - 1 - y;
                    self.print_screen();
                },
                '\x01' => { // Ctrl A -> Go to beginning of line
                    x = 0;
                },
                '\x05' => { // Ctrl E -> Go to end of line
                    let line_length = self.lines[self.offset + y].len();
                    if line_length > 0 {
                        x = cmp::min(line_length, self.width() - 1);
                    }
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

                        let mut line = self.lines[self.offset + y].clone();
                        line.truncate(self.width());
                        kernel::vga::clear_row();
                        print!("{}", line);
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
                    if !c.is_ascii() || !kernel::vga::is_printable(c as u8) {
                        continue;
                    }

                    let line = self.lines[self.offset + y].clone();
                    let (before_cursor, after_cursor) = line.split_at(x);
                    self.lines[self.offset + y].clear();
                    self.lines[self.offset + y].push_str(before_cursor);
                    self.lines[self.offset + y].push(c);
                    self.lines[self.offset + y].push_str(after_cursor);

                    let mut line = self.lines[self.offset + y].clone();
                    line.truncate(self.width());
                    kernel::vga::clear_row();
                    print!("{}", line);
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
        kernel::vga::screen_height() - 1 // Leave out one line for status line
    }

    fn width(&self) -> usize {
        kernel::vga::screen_width()
    }
}
