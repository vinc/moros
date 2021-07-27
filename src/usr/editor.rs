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
    x: usize,
    y: usize,
    dx: usize, // Horizontal offset from the start
    dy: usize, // Vertical offset from the top
    config: EditorConfig,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        let x = 0;
        let y = 0;
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

        Self { file, pathname, lines, x, y, dx, dy, config }
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
            file.write(contents.as_bytes()).unwrap();
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
        print!("\x1b[{};1H", self.rows() + 1); // Move cursor to the bottom of the screen
        print!("{}{:cols$}{}", color, status, reset, cols = self.cols());
        print!("\x1b[{};{}H", self.y + 1, self.x + 1); // Move cursor back
    }

    fn print_screen(&mut self) {
        let mut rows: Vec<String> = Vec::new();
        let a = self.dy;
        let b = self.dy + self.rows();
        for y in a..b {
            rows.push(self.render_line(y));
        }
        println!("\x1b[1;1H{}", rows.join(""));

        let status = format!("Editing '{}'", self.pathname);
        self.print_status(&status, "LightGray");
    }

    fn render_line(&self, y: usize) -> String {
        // Render line into a row of the screen, or an empty row when past eof
        let line = if y < self.lines.len() { &self.lines[y] } else { "" };

        let mut row = format!("{:cols$}", line, cols = self.dx);
        let n = self.dx + self.cols();
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
            '\t'      => Some(" ".repeat(self.config.tab_size)),
            _         => None,
        }
    }

    pub fn run(&mut self) -> usr::shell::ExitCode {
        print!("\x1b[2J"); // Clear screen
        self.print_screen();
        print!("\x1b[1;1H"); // Move cursor to the top of the screen

        let mut escape = false;
        let mut csi = false;
        loop {
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
                '\0' => {
                    continue;
                }
                '\x11' => { // Ctrl Q
                    // TODO: Warn if modifications have not been saved
                    print!("\x1b[2J"); // Clear screen
                    break;
                },
                '\x17' => { // Ctrl W
                    self.save();
                },
                '\x18' => { // Ctrl X
                    let res = self.save();
                    print!("\x1b[2J"); // Clear screen
                    return res;
                },
                '\n' => { // Newline
                    let line = self.lines[self.dy + self.y].split_off(self.dx + self.x);
                    self.lines.insert(self.dy + self.y + 1, line);
                    if self.y == self.rows() - 1 {
                        self.dy += 1;
                    } else {
                        self.y += 1;
                    }
                    self.x = 0;
                    self.dx = 0;
                    self.print_screen();
                },
                'A' if csi => { // Arrow up
                    if self.y > 0 {
                        self.y -= 1
                    } else if self.dy > 0 {
                        self.dy -= 1;
                        self.print_screen();
                    }
                    self.x = self.next_pos(self.x, self.y);
                },
                'B' if csi => { // Arrow down
                    let is_eof = self.dy + self.y == self.lines.len() - 1;
                    let is_bottom = self.y == self.rows() - 1;
                    if self.y < cmp::min(self.rows(), self.lines.len() - 1) {
                        if is_bottom || is_eof {
                            if !is_eof {
                                self.dy += 1;
                                self.print_screen();
                            }
                        } else {
                            self.y += 1;
                        }
                        self.x = self.next_pos(self.x, self.y);
                    }
                },
                'C' if csi => { // Arrow right
                    let line = &self.lines[self.dy + self.y];
                    if line.is_empty() || self.x + self.dx >= line.len() {
                        continue
                    } else if self.x == self.cols() - 1 {
                        self.x = self.dx;
                        self.dx += self.cols();
                        self.print_screen();
                    } else {
                        self.x += 1;
                    }
                },
                'D' if csi => { // Arrow left
                    if self.x + self.dx == 0 {
                        continue;
                    } else if self.x == 0 {
                        self.x = self.dx - 1;
                        self.dx -= self.cols();
                        self.print_screen();
                        self.x = self.next_pos(self.x, self.y);
                    } else {
                        self.x -= 1;
                    }
                },
                '\x14' => { // Ctrl T -> Go to top of file
                    self.x = 0;
                    self.y = 0;
                    self.dx = 0;
                    self.dy = 0;
                    self.print_screen();
                },
                '\x02' => { // Ctrl B -> Go to bottom of file
                    self.x = 0;
                    self.y = cmp::min(self.rows(), self.lines.len()) - 1;
                    self.dx = 0;
                    self.dy = self.lines.len() - 1 - self.y;
                    self.print_screen();
                },
                '\x01' => { // Ctrl A -> Go to beginning of line
                    self.x = 0;
                    self.dx = 0;
                    self.print_screen();
                },
                '\x05' => { // Ctrl E -> Go to end of line
                    let n = self.lines[self.dy + self.y].len();
                    let w = self.cols();
                    self.x = n % w;
                    self.dx = w * (n / w);
                    self.print_screen();
                },
                '\x08' => { // Backspace
                    if self.dx + self.x > 0 { // Remove char from line
                        let line = self.lines[self.dy + self.y].clone();
                        let pos = self.dx + self.x - 1;
                        let (before, mut after) = line.split_at(pos);
                        if !after.is_empty() {
                            after = &after[1..];
                        }
                        self.lines[self.dy + self.y].clear();
                        self.lines[self.dy + self.y].push_str(before);
                        self.lines[self.dy + self.y].push_str(after);

                        if self.x == 0 {
                            self.dx -= self.cols();
                            self.x = self.cols() - 1;
                            self.print_screen();
                        } else {
                            self.x -= 1;
                            let line = self.render_line(self.dy + self.y);
                            print!("\x1b[2K\x1b[1G{}", line);
                        }
                    } else { // Remove newline from previous line
                        if self.y == 0 && self.dy == 0 {
                            continue;
                        }

                        // Move cursor below the end of the previous line
                        let n = self.lines[self.dy + self.y - 1].len();
                        let w = self.cols();
                        self.x = n % w;
                        self.dx = w * (n / w);

                        // Move line to the end of the previous line
                        let line = self.lines.remove(self.dy + self.y);
                        self.lines[self.dy + self.y - 1].push_str(&line);

                        // Move cursor up to the previous line
                        if self.y > 0 {
                            self.y -= 1;
                        } else {
                            self.dy -= 1;
                        }

                        self.print_screen();
                    }
                },
                '\x7f' => { // Delete
                    let n = self.lines[self.dy + self.y].len();
                    if self.dx + self.x >= n { // Remove newline from line
                        let line = self.lines.remove(self.dy + self.y + 1);
                        self.lines[self.dy + self.y].push_str(&line);
                        self.print_screen();
                    } else { // Remove char from line
                        self.lines[self.dy + self.y].remove(self.dx + self.x);
                        let line = self.render_line(self.dy + self.y);
                        print!("\x1b[2K\x1b[1G{}", line);
                    }
                },
                c => {
                    if let Some(s) = self.render_char(c) {
                        let line = self.lines[self.dy + self.y].clone();
                        let (before, after) = line.split_at(self.dx + self.x);
                        self.lines[self.dy + self.y].clear();
                        self.lines[self.dy + self.y].push_str(before);
                        self.lines[self.dy + self.y].push_str(&s);
                        self.lines[self.dy + self.y].push_str(after);

                        self.x += s.len();
                        if self.x >= self.cols() {
                            self.dx += self.cols();
                            self.x -= self.dx;
                            self.print_screen();
                        } else {
                            let line = self.render_line(self.dy + self.y);
                            print!("\x1b[2K\x1b[1G{}", line);
                        }
                    }
                },
            }
            escape = false;
            csi = false;
            print!("\x1b[{};{}H", self.y + 1, self.x + 1);
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

    fn rows(&self) -> usize {
        sys::console::rows() - 1 // Leave out one line for status line
    }

    fn cols(&self) -> usize {
        sys::console::cols()
    }
}

fn truncated_line_indicator() -> String {
    let color = Style::color("Black").with_background("LightGray");
    let reset = Style::reset();
    format!("{}>{}", color, reset)
}
