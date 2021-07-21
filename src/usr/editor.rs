use crate::{sys, usr};
use crate::api::console::Style;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 2 {
        return usr::shell::ExitCode::CommandError;
    }

    let pathname = args[1];
    let mut editor = Editor::new(pathname);
    editor.run()
}

struct EditorConfig {
    tab_size: usize,
}

pub struct Editor {
    file: Option<sys::fs::File>,
    pathname: String,
    lines: Vec<String>,
    dx: usize, // Horizontal offset
    dy: usize, // Vertical offset
    config: EditorConfig,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        let dx = 0;
        let dy = 0;
        let mut lines = Vec::new();
        let config = EditorConfig { tab_size: 4 };

        let file = match sys::fs::File::open(pathname) {
            Some(mut file) => {
                let contents = file.read_to_string();
                for line in contents.split('\n') {
                    lines.push(line.into());
                }
                Some(file)
            },
            None => {
                lines.push(String::new());
                sys::fs::File::create(pathname)
            }
        };

        let pathname = pathname.into();

        Self { file, pathname, lines, dx, dy, config }
    }

    pub fn save(&mut self) -> usr::shell::ExitCode {
        let mut contents = String::new();
        let n = self.lines.len();
        for i in 0..n {
            contents.push_str(&self.lines[i]);
            if i < n - 1 {
                contents.push('\n');
            }
        }

        if let Some(file) = &mut self.file {
            file.seek(sys::fs::SeekFrom::Start(0)).unwrap();
            file.write(&contents.as_bytes()).unwrap();
            let status = format!("Wrote {}L to '{}'", n, self.pathname);
            self.print_status(&status, "Yellow");
            usr::shell::ExitCode::CommandSuccessful
        } else {
            let status = format!("Could not write to '{}'", self.pathname);
            self.print_status(&status, "LightRed");
            usr::shell::ExitCode::CommandError
        }
    }

    fn print_status(&mut self, status: &str, background: &str) {
        let color = Style::color("Black").with_background(background);
        let reset = Style::reset();
        let (x, y) = sys::vga::cursor_position();
        sys::vga::set_writer_position(0, self.height());
        print!("{}{:width$}{}", color, status, reset, width = self.width());
        sys::vga::set_writer_position(x, y);
        sys::vga::set_cursor_position(x, y);
    }

    fn print_screen(&mut self) {
        let mut rows: Vec<String> = Vec::new();
        let a = self.dy;
        let b = self.dy + self.height();
        for y in a..b {
            rows.push(self.render_line(y));
        }
        sys::vga::set_writer_position(0, 0);
        print!("{}", rows.join("\n"));

        let status = format!("Editing '{}'", self.pathname);
        self.print_status(&status, "LightGray");
    }

    fn render_line(&self, y: usize) -> String {
        // Render line into a row of the screen, or an empty row when past eof
        let line = if y < self.lines.len() { &self.lines[y] } else { "" };

        let mut row = format!("{:width$}", line, width = self.dx);
        let n = self.dx + self.width();
        if row.len() > n {
            row.truncate(n - 1);
            row.push_str(&truncated_line_indicator());
        } else {
            row.push_str(&" ".repeat(n - row.len()));
        }
        row[self.dx..].to_string()
    }

    fn render_char(&self, c: char) -> Option<String> {
        match c {
            '!'..='~' => Some(c.to_string()), // graphic char
            ' '       => Some(" ".to_string()),
            '\t'      => Some(" ".repeat(self.config.tab_size).to_string()),
            _         => None,
        }
    }

    pub fn run(&mut self) -> usr::shell::ExitCode {
        sys::vga::clear_screen();
        self.print_screen();
        sys::vga::set_cursor_position(0, 0);
        sys::vga::set_writer_position(0, 0);

        let mut escape = false;
        let mut csi = false;
        loop {
            let (mut x, mut y) = sys::vga::cursor_position();
            let c = sys::console::get_char();
            match c {
                '\x1B' => { // ESC
                    escape = true;
                    continue;
                },
                '[' if escape => {
                    csi = true;
                    continue;
                },
                _ => {},
            }
            match c {
                '\0' => {
                    continue;
                }
                '\x11' => { // Ctrl Q
                    // TODO: Warn if modifications have not been saved
                    sys::vga::clear_screen();
                    break;
                },
                '\x17' => { // Ctrl W
                    self.save();
                },
                '\x18' => { // Ctrl X
                    let res = self.save();
                    sys::vga::clear_screen();
                    return res;
                },
                '\n' => { // Newline
                    let line = self.lines[self.dy + y].split_off(self.dx + x);
                    self.lines.insert(self.dy + y + 1, line);
                    if y == self.height() - 1 {
                        self.dy += 1;
                    } else {
                        y += 1;
                    }
                    x = 0;
                    self.dx = 0;
                    self.print_screen();
                },
                'A' if csi => { // Arrow up
                    if y > 0 {
                        y -= 1
                    } else {
                        if self.dy > 0 {
                            self.dy -= 1;
                            self.print_screen();
                        }
                    }
                    x = self.next_pos(x, y);
                },
                'B' if csi => { // Arrow down
                    let is_eof = self.dy + y == self.lines.len() - 1;
                    let is_bottom = y == self.height() - 1;
                    if y < cmp::min(self.height(), self.lines.len() - 1) {
                        if is_bottom || is_eof {
                            if !is_eof {
                                self.dy += 1;
                                self.print_screen();
                            }
                        } else {
                            y += 1;
                        }
                        x = self.next_pos(x, y);
                    }
                },
                'C' if csi => { // Arrow right
                    let line = &self.lines[self.dy + y];
                    if line.len() == 0 || x + self.dx >= line.len() {
                        continue
                    } else if x == self.width() - 1 {
                        x = self.dx;
                        self.dx += self.width();
                        self.print_screen();
                    } else {
                        x += 1;
                    }
                },
                'D' if csi => { // Arrow left
                    if x + self.dx == 0 {
                        continue;
                    } else if x == 0 {
                        x = self.dx - 1;
                        self.dx -= self.width();
                        self.print_screen();
                        x = self.next_pos(x, y);
                    } else {
                        x -= 1;
                    }
                },
                '\x14' => { // Ctrl T -> Go to top of file
                    x = 0;
                    y = 0;
                    self.dx = 0;
                    self.dy = 0;
                    self.print_screen();
                },
                '\x02' => { // Ctrl B -> Go to bottom of file
                    x = 0;
                    y = cmp::min(self.height(), self.lines.len()) - 1;
                    self.dx = 0;
                    self.dy = self.lines.len() - 1 - y;
                    self.print_screen();
                },
                '\x01' => { // Ctrl A -> Go to beginning of line
                    x = 0;
                    self.dx = 0;
                    self.print_screen();
                },
                '\x05' => { // Ctrl E -> Go to end of line
                    let n = self.lines[self.dy + y].len();
                    let w = self.width();
                    x = n % w;
                    self.dx = w * (n / w);
                    self.print_screen();
                },
                '\x08' => { // Backspace
                    if self.dx + x > 0 { // Remove char from line
                        let line = self.lines[self.dy + y].clone();
                        let pos = self.dx + x - 1;
                        let (before, mut after) = line.split_at(pos);
                        if after.len() > 0 {
                            after = &after[1..];
                        }
                        self.lines[self.dy + y].clear();
                        self.lines[self.dy + y].push_str(before);
                        self.lines[self.dy + y].push_str(after);

                        if x == 0 {
                            self.dx -= self.width();
                            x = self.width() - 1;
                            self.print_screen();
                        } else {
                            x -= 1;
                            let line = self.render_line(self.dy + y);
                            sys::vga::clear_row();
                            print!("{}", line);
                        }
                    } else { // Remove newline from previous line
                        if y == 0 && self.dy == 0 {
                            continue;
                        }

                        // Move cursor below the end of the previous line
                        let n = self.lines[self.dy + y - 1].len();
                        let w = self.width();
                        x = n % w;
                        self.dx = w * (n / w);

                        // Move line to the end of the previous line
                        let line = self.lines.remove(self.dy + y);
                        self.lines[self.dy + y - 1].push_str(&line);

                        // Move cursor up to the previous line
                        if y > 0 {
                            y -= 1;
                        } else {
                            self.dy -= 1;
                        }

                        self.print_screen();
                    }
                },
                '\x7f' => { // Delete
                    let n = self.lines[self.dy + y].len();
                    if self.dx + x >= n { // Remove newline from line
                        let line = self.lines.remove(self.dy + y + 1);
                        self.lines[self.dy + y].push_str(&line);
                        self.print_screen();
                    } else { // Remove char from line
                        self.lines[self.dy + y].remove(self.dx + x);
                        let line = self.render_line(self.dy + y);
                        sys::vga::clear_row();
                        print!("{}", line);
                    }
                },
                c => {
                    if let Some(s) = self.render_char(c) {
                        let line = self.lines[self.dy + y].clone();
                        let (before, after) = line.split_at(self.dx + x);
                        self.lines[self.dy + y].clear();
                        self.lines[self.dy + y].push_str(before);
                        self.lines[self.dy + y].push_str(&s);
                        self.lines[self.dy + y].push_str(after);

                        x += s.len();
                        if x >= self.width() {
                            self.dx += self.width();
                            x -= self.dx;
                            self.print_screen();
                        } else {
                            let line = self.render_line(self.dy + y);
                            sys::vga::clear_row();
                            print!("{}", line);
                        }
                    }
                },
            }
            escape = false;
            csi = false;
            sys::vga::set_cursor_position(x, y);
            sys::vga::set_writer_position(x, y);
        }
        usr::shell::ExitCode::CommandSuccessful
    }

    // Move cursor past end of line to end of line or left of the screen
    fn next_pos(&self, x: usize, y: usize) -> usize {
        let eol = self.lines[self.dy + y].len();
        if eol <= self.dx + x {
            if eol <= self.dx {
                0
            } else {
                eol - 1
            }
        } else {
            x
        }
    }

    fn height(&self) -> usize {
        sys::vga::screen_height() - 1 // Leave out one line for status line
    }

    fn width(&self) -> usize {
        sys::vga::screen_width()
    }
}

fn truncated_line_indicator() -> String {
    let color = Style::color("Black").with_background("LightGray");
    let reset = Style::reset();
    format!("{}>{}", color, reset)
}
