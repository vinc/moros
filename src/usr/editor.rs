use crate::{sys, usr};
use crate::api::{console, fs, io};
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

        match fs::read_to_string(pathname) {
            Ok(contents) => {
                for line in contents.split('\n') {
                    lines.push(line.into());
                }
            },
            Err(_) => {
                lines.push(String::new());
            }
        };

        let pathname = pathname.into();

        Self { pathname, lines, x, y, dx, dy, config }
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

        if fs::write(&self.pathname, contents.as_bytes()).is_ok() {
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
        //print!("\x1b[{};{}H", self.y + 1, self.x + 1); // Move cursor back
    }

    fn print_editing_status(&mut self) {
        let max = 50;
        let mut path = self.pathname.clone();
        if self.pathname.chars().count() > max {
            path.truncate(max - 3);
            path.push_str("...");
        }
        let start = format!("Editing '{}'", path);

        let x = self.dx + self.x + 1;
        let y = self.dy + self.y + 1;
        let n = y * 100 / self.lines.len();
        let end = format!("{},{} {:3}%", y, x, n);

        let width = self.cols() - start.chars().count();
        let status = format!("{}{:>width$}", start, end, width = width);

        self.print_status(&status, "LightGray");
    }

    fn print_screen(&mut self) {
        let mut rows: Vec<String> = Vec::new();
        let a = self.dy;
        let b = self.dy + self.rows();
        for y in a..b {
            rows.push(self.render_line(y));
        }
        println!("\x1b[1;1H{}", rows.join("\n"));
    }

    fn render_line(&self, y: usize) -> String {
        // Render line into a row of the screen, or an empty row when past eof
        let line = if y < self.lines.len() { &self.lines[y] } else { "" };

        let mut row: Vec<char> = format!("{:cols$}", line, cols = self.dx).chars().collect();
        let n = self.dx + self.cols();
        let after = if row.len() > n {
            row.truncate(n - 1);
            truncated_line_indicator()
        } else {
            " ".repeat(n - row.len())
        };
        row.extend(after.chars());
        row[self.dx..].iter().collect()
    }

    fn render_char(&self, c: char) -> Option<String> {
        match c {
            '\t'                          => Some(" ".repeat(self.config.tab_size)),
            c if console::is_printable(c) => Some(c.to_string()),
            _                             => None,
        }
    }

    pub fn run(&mut self) -> usr::shell::ExitCode {
        print!("\x1b[2J\x1b[1;1H"); // Clear screen and move cursor to top
        self.print_screen();
        self.print_editing_status();
        print!("\x1b[1;1H"); // Move cursor to the top of the screen

        let mut escape = false;
        let mut csi = false;
        loop {
            let c = io::stdin().read_char().unwrap_or('\0');
            print!("\x1b[?25l"); // Disable cursor
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
                    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move cursor to top
                    print!("\x1b[?25h"); // Enable cursor
                    break;
                },
                '\x17' => { // Ctrl W
                    self.save();
                    print!("\x1b[?25h"); // Enable cursor
                    continue;
                },
                '\x18' => { // Ctrl X
                    let res = self.save();
                    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move cursor to top
                    print!("\x1b[?25h"); // Enable cursor
                    return res;
                },
                '\n' => { // Newline
                    let y = self.dy + self.y;
                    let old_line = self.lines[y].clone();
                    let mut row: Vec<char> = old_line.chars().collect();
                    let new_line = row.split_off(self.dx + self.x).into_iter().collect();
                    self.lines[y] = row.into_iter().collect();
                    self.lines.insert(y + 1, new_line);
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
                    if line.is_empty() || self.x + self.dx >= line.chars().count() {
                        print!("\x1b[?25h"); // Enable cursor
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
                        print!("\x1b[?25h"); // Enable cursor
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
                    let n = self.lines[self.dy + self.y].chars().count();
                    let w = self.cols();
                    self.x = n % w;
                    self.dx = w * (n / w);
                    self.print_screen();
                },
                '\x08' => { // Backspace
                    let y = self.dy + self.y;
                    if self.dx + self.x > 0 { // Remove char from line

                        let mut row: Vec<char> = self.lines[y].chars().collect();
                        row.remove(self.dx + self.x - 1);
                        self.lines[y] = row.into_iter().collect();

                        if self.x == 0 {
                            self.dx -= self.cols();
                            self.x = self.cols() - 1;
                            self.print_screen();
                        } else {
                            self.x -= 1;
                            let line = self.render_line(y);
                            print!("\x1b[2K\x1b[1G{}", line);
                        }
                    } else { // Remove newline from previous line
                        if self.y == 0 && self.dy == 0 {
                            print!("\x1b[?25h"); // Enable cursor
                            continue;
                        }

                        // Move cursor below the end of the previous line
                        let n = self.lines[y - 1].chars().count();
                        let w = self.cols();
                        self.x = n % w;
                        self.dx = w * (n / w);

                        // Move line to the end of the previous line
                        let line = self.lines.remove(y);
                        self.lines[y - 1].push_str(&line);

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
                    let y = self.dy + self.y;
                    let n = self.lines[y].chars().count();
                    if self.dx + self.x >= n { // Remove newline from line
                        let line = self.lines.remove(y + 1);
                        self.lines[y].push_str(&line);
                        self.print_screen();
                    } else { // Remove char from line
                        self.lines[y].remove(self.dx + self.x);
                        let line = self.render_line(y);
                        print!("\x1b[2K\x1b[1G{}", line);
                    }
                },
                c => {
                    if let Some(s) = self.render_char(c) {
                        let y = self.dy + self.y;
                        let mut row: Vec<char> = self.lines[y].chars().collect();
                        for c in s.chars() {
                            row.insert(self.dx + self.x, c);
                            self.x += 1;
                        }
                        self.lines[y] = row.into_iter().collect();
                        if self.x >= self.cols() {
                            self.dx += self.cols();
                            self.x -= self.cols();
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
            self.print_editing_status();
            print!("\x1b[{};{}H", self.y + 1, self.x + 1);
            print!("\x1b[?25h"); // Enable cursor
        }
        usr::shell::ExitCode::CommandSuccessful
    }

    // Move cursor past end of line to end of line or left of the screen
    fn next_pos(&self, x: usize, y: usize) -> usize {
        let eol = self.lines[self.dy + y].chars().count();
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
