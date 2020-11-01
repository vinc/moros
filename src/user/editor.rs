use crate::{kernel, print, user};
use crate::kernel::console::Style;
use alloc::format;
use alloc::string::{String, ToString};
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

struct EditorConfig {
    tab_size: usize,
}

pub struct Editor {
    file: Option<kernel::fs::File>,
    pathname: String,
    lines: Vec<String>,
    dy: usize, // Vertical offset
    config: EditorConfig,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        let dy = 0;
        let mut lines = Vec::new();
        let config = EditorConfig { tab_size: 4 };

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

        Self { file, pathname, lines, dy, config }
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

        if let Some(file) = &mut self.file {
            file.write(&contents.as_bytes()).unwrap();
            let status = format!("Wrote {}L to '{}'", n, self.pathname);
            self.print_status(&status, "Yellow");
            user::shell::ExitCode::CommandSuccessful
        } else {
            let status = format!("Could not write to '{}'", self.pathname);
            self.print_status(&status, "LightRed");
            user::shell::ExitCode::CommandError
        }
    }

    fn print_status(&mut self, status: &str, background: &str) {
        let color = Style::color("Black").with_background(background);
        let reset = Style::reset();
        let (x, y) = kernel::vga::cursor_position();
        kernel::vga::set_writer_position(0, self.height());
        print!("{}{:width$}{}", color, status, reset, width = self.width());
        kernel::vga::set_writer_position(x, y);
        kernel::vga::set_cursor_position(x, y);
    }

    fn print_screen(&mut self) {
        let mut lines: Vec<String> = Vec::new();
        let a = self.dy;
        let b = cmp::min(self.lines.len(), self.dy + self.height());
        for y in a..b {
            lines.push(self.render_line(y)); // TODO: Use `dx .. dx + n`
        }
        kernel::vga::set_writer_position(0, 0);
        print!("{}", lines.join("\n"));
        let status = format!("Editing '{}'", self.pathname);
        self.print_status(&status, "LightGray");
    }

    fn render_line(&self, y: usize) -> String {
        let n = self.width();
        let mut line = self.lines[y].to_string();
        if line.len() > n {
            line.truncate(n - 1);
            line.push_str(&truncated_line_indicator());
        } else {
            line.push_str(&" ".repeat(n - line.len()));
        }
        line
    }

    fn render_char(&self, c: char) -> Option<String> {
        match c {
            '!'..='~' => Some(c.to_string()), // graphic char
            ' '       => Some(" ".to_string()),
            '\t'      => Some(" ".repeat(self.config.tab_size).to_string()),
            _         => None,
        }
    }

    pub fn run(&mut self) -> user::shell::ExitCode {
        kernel::vga::clear_screen();
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
                    let new_line = self.lines[self.dy + y].split_off(x);
                    self.lines.insert(self.dy + y + 1, new_line);
                    if y == self.height() - 1 {
                        self.dy += 1;
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
                        if self.dy > 0 {
                            self.dy -= 1;
                            self.print_screen();
                        }
                    }
                    x = cmp::min(x, self.lines[self.dy + y].len());
                },
                '↓' => { // Arrow down
                    let is_bottom = y == self.height() - 1;
                    let is_eof = self.dy + y == self.lines.len() - 1;
                    if y < cmp::min(self.height(), self.lines.len() - 1) {
                        if is_bottom || is_eof {
                            if !is_eof {
                                self.dy += 1;
                                self.print_screen();
                            }
                        } else {
                            y += 1;
                        }
                        x = cmp::min(x, self.lines[self.dy + y].len());
                    }
                },
                '←' => { // Arrow left
                    if x == 0 {
                        continue;
                    }
                    x -= 1;
                },
                '→' => { // Arrow right
                    let line = &self.lines[self.dy + y];
                    if x == cmp::min(self.width() - 1, line.len()) {
                        continue;
                    }
                    x += 1;
                },
                '\x14' => { // Ctrl T -> Go to top of file
                    x = 0;
                    y = 0;
                    self.dy = 0;
                    self.print_screen();
                },
                '\x02' => { // Ctrl B -> Go to bottom of file
                    x = 0;
                    y = cmp::min(self.height(), self.lines.len()) - 1;
                    self.dy = self.lines.len() - 1 - y;
                    self.print_screen();
                },
                '\x01' => { // Ctrl A -> Go to beginning of line
                    x = 0;
                },
                '\x05' => { // Ctrl E -> Go to end of line
                    let line_length = self.lines[self.dy + y].len();
                    if line_length > 0 {
                        x = cmp::min(line_length, self.width() - 1);
                    }
                },
                '\x08' => { // Backspace
                    if x > 0 { // Remove char from line
                        let line = self.lines[self.dy + y].clone();
                        let (before, mut after) = line.split_at(x - 1);
                        if after.len() > 0 {
                            after = &after[1..];
                        }
                        self.lines[self.dy + y].clear();
                        self.lines[self.dy + y].push_str(before);
                        self.lines[self.dy + y].push_str(after);

                        let line = self.render_line(self.dy + y);
                        kernel::vga::clear_row();
                        print!("{}", line);
                        x -= 1;
                    } else { // Remove newline char from previous line
                        if y == 0 && self.dy == 0 {
                            continue;
                        }

                        x = self.lines[self.dy + y - 1].len();
                        let line = self.lines.remove(self.dy + y);
                        self.lines[self.dy + y - 1].push_str(&line);
                        if y > 0 {
                            y -= 1;
                        } else {
                            self.dy -= 1;
                        }
                        self.print_screen();
                    }
                },
                c => {
                    if let Some(s) = self.render_char(c) {
                        let line = self.lines[self.dy + y].clone();
                        let (before_cursor, after_cursor) = line.split_at(x);
                        self.lines[self.dy + y].clear();
                        self.lines[self.dy + y].push_str(before_cursor);
                        self.lines[self.dy + y].push_str(&s);
                        self.lines[self.dy + y].push_str(after_cursor);

                        let line = self.render_line(self.dy + y);
                        kernel::vga::clear_row();
                        print!("{}", line);

                        x += s.len();
                        x = cmp::min(x, self.lines[self.dy + y].len());
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

fn truncated_line_indicator() -> String {
    let color = Style::color("Black").with_background("LightGray");
    let reset = Style::reset();
    format!("{}>{}", color, reset)
}
