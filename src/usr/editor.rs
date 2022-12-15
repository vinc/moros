use crate::sys;
use crate::api::{console, fs, io};
use crate::api::console::Style;
use crate::api::process::ExitCode;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 2 {
        return Err(ExitCode::UsageError);
    }

    let pathname = args[1];
    let mut editor = Editor::new(pathname);
    editor.run()
}

struct EditorConfig {
    tab_size: usize,
}

struct Coords {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    pathname: String,
    clipboard: Vec<String>,
    lines: Vec<String>,
    cursor: Coords,
    offset: Coords,
    config: EditorConfig,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        let cursor = Coords { x: 0, y: 0 };
        let offset = Coords { x: 0, y: 0 };
        let clipboard = Vec::new();
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

        Self { pathname, clipboard, lines, cursor, offset, config }
    }

    pub fn save(&mut self) -> Result<(), ExitCode> {
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
            Ok(())
        } else {
            let status = format!("Could not write to '{}'", self.pathname);
            self.print_status(&status, "LightRed");
            Err(ExitCode::Failure)
        }
    }

    fn print_status(&mut self, status: &str, background: &str) {
        let color = Style::color("Black").with_background(background);
        let reset = Style::reset();
        print!("\x1b[{};1H", self.rows() + 1); // Move cursor to the bottom of the screen
        print!("{}{:cols$}{}", color, status, reset, cols = self.cols());
        print!("\x1b[{};{}H", self.cursor.y + 1, self.cursor.x + 1); // Move cursor back
    }

    fn print_editing_status(&mut self) {
        let max = 50;
        let mut path = self.pathname.clone();
        if self.pathname.chars().count() > max {
            path.truncate(max - 3);
            path.push_str("...");
        }
        let start = format!("Editing '{}'", path);

        let x = self.offset.x + self.cursor.x + 1;
        let y = self.offset.y + self.cursor.y + 1;
        let n = y * 100 / self.lines.len();
        let end = format!("{},{} {:3}%", y, x, n);

        let width = self.cols() - start.chars().count();
        let status = format!("{}{:>width$}", start, end, width = width);

        self.print_status(&status, "LightGray");
    }

    fn print_screen(&mut self) {
        let mut rows: Vec<String> = Vec::new();
        let a = self.offset.y;
        let b = self.offset.y + self.rows();
        for y in a..b {
            rows.push(self.render_line(y));
        }
        println!("\x1b[1;1H{}", rows.join("\n"));
    }

    fn render_line(&self, y: usize) -> String {
        // Render line into a row of the screen, or an empty row when past eof
        let line = if y < self.lines.len() { &self.lines[y] } else { "" };

        let mut row: Vec<char> = format!("{:cols$}", line, cols = self.offset.x).chars().collect();
        let n = self.offset.x + self.cols();
        let after = if row.len() > n {
            row.truncate(n - 1);
            truncated_line_indicator()
        } else {
            " ".repeat(n - row.len())
        };
        row.extend(after.chars());
        row[self.offset.x..].iter().collect()
    }

    fn render_char(&self, c: char) -> Option<String> {
        match c {
            '\t'                          => Some(" ".repeat(self.config.tab_size)),
            c if console::is_printable(c) => Some(c.to_string()),
            _                             => None,
        }
    }

    pub fn run(&mut self) -> Result<(), ExitCode> {
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
                '\x11' | '\x03' => { // Ctrl Q or Ctrl C
                    // TODO: Warn if modifications have not been saved
                    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move cursor to top
                    print!("\x1b[?25h"); // Enable cursor
                    break;
                },
                '\x17' => { // Ctrl W
                    self.save().ok();
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
                    let y = self.offset.y + self.cursor.y;
                    let old_line = self.lines[y].clone();
                    let mut row: Vec<char> = old_line.chars().collect();
                    let new_line = row.split_off(self.offset.x + self.cursor.x).into_iter().collect();
                    self.lines[y] = row.into_iter().collect();
                    self.lines.insert(y + 1, new_line);
                    if self.cursor.y == self.rows() - 1 {
                        self.offset.y += 1;
                    } else {
                        self.cursor.y += 1;
                    }
                    self.cursor.x = 0;
                    self.offset.x = 0;
                    self.print_screen();
                },
                'A' if csi => { // Arrow up
                    if self.cursor.y > 0 {
                        self.cursor.y -= 1
                    } else if self.offset.y > 0 {
                        self.offset.y -= 1;
                        self.print_screen();
                    }
                    self.cursor.x = self.next_pos(self.cursor.x, self.cursor.y);
                },
                'B' if csi => { // Arrow down
                    let is_eof = self.offset.y + self.cursor.y == self.lines.len() - 1;
                    let is_bottom = self.cursor.y == self.rows() - 1;
                    if self.cursor.y < cmp::min(self.rows(), self.lines.len() - 1) {
                        if is_bottom || is_eof {
                            if !is_eof {
                                self.offset.y += 1;
                                self.print_screen();
                            }
                        } else {
                            self.cursor.y += 1;
                        }
                        self.cursor.x = self.next_pos(self.cursor.x, self.cursor.y);
                    }
                },
                'C' if csi => { // Arrow right
                    let line = &self.lines[self.offset.y + self.cursor.y];
                    if line.is_empty() || self.cursor.x + self.offset.x >= line.chars().count() {
                        print!("\x1b[?25h"); // Enable cursor
                        continue
                    } else if self.cursor.x == self.cols() - 1 {
                        self.cursor.x = self.offset.x;
                        self.offset.x += self.cols();
                        self.print_screen();
                    } else {
                        self.cursor.x += 1;
                    }
                },
                'D' if csi => { // Arrow left
                    if self.cursor.x + self.offset.x == 0 {
                        print!("\x1b[?25h"); // Enable cursor
                        continue;
                    } else if self.cursor.x == 0 {
                        self.cursor.x = self.offset.x - 1;
                        self.offset.x -= self.cols();
                        self.print_screen();
                        self.cursor.x = self.next_pos(self.cursor.x, self.cursor.y);
                    } else {
                        self.cursor.x -= 1;
                    }
                },
                'Z' if csi => { // Backtab (Shift + Tab)
                    // Do nothing
                },
                '\x14' => { // Ctrl T -> Go to top of file
                    self.cursor.x = 0;
                    self.cursor.y = 0;
                    self.offset.x = 0;
                    self.offset.y = 0;
                    self.print_screen();
                },
                '\x02' => { // Ctrl B -> Go to bottom of file
                    self.cursor.x = 0;
                    self.cursor.y = cmp::min(self.rows(), self.lines.len()) - 1;
                    self.offset.x = 0;
                    self.offset.y = self.lines.len() - 1 - self.cursor.y;
                    self.print_screen();
                },
                '\x01' => { // Ctrl A -> Go to beginning of line
                    self.cursor.x = 0;
                    self.offset.x = 0;
                    self.print_screen();
                },
                '\x05' => { // Ctrl E -> Go to end of line
                    let n = self.lines[self.offset.y + self.cursor.y].chars().count();
                    let w = self.cols();
                    self.cursor.x = n % w;
                    self.offset.x = w * (n / w);
                    self.print_screen();
                },
                '\x04' => { // Ctrl D -> Delete (cut) line
                    let i = self.offset.y + self.cursor.y;
                    self.clipboard.push(self.lines.remove(i));
                    if self.lines.is_empty() {
                        self.lines.push(String::new());
                    }

                    if i >= self.lines.len() {
                        // Move cursor up to the previous line
                        if self.cursor.y > 0 {
                            self.cursor.y -= 1;
                        } else if self.offset.y > 0 {
                            self.offset.y -= 1;
                        }
                    }
                    self.cursor.x = 0;
                    self.offset.x = 0;

                    self.print_screen();
                },
                '\x08' => { // Backspace
                    let y = self.offset.y + self.cursor.y;
                    if self.offset.x + self.cursor.x > 0 { // Remove char from line
                        let mut row: Vec<char> = self.lines[y].chars().collect();
                        row.remove(self.offset.x + self.cursor.x - 1);
                        self.lines[y] = row.into_iter().collect();

                        if self.cursor.x == 0 {
                            self.offset.x -= self.cols();
                            self.cursor.x = self.cols() - 1;
                            self.print_screen();
                        } else {
                            self.cursor.x -= 1;
                            let line = self.render_line(y);
                            print!("\x1b[2K\x1b[1G{}", line);
                        }
                    } else { // Remove newline from previous line
                        if self.cursor.y == 0 && self.offset.y == 0 {
                            print!("\x1b[?25h"); // Enable cursor
                            continue;
                        }

                        // Move cursor below the end of the previous line
                        let n = self.lines[y - 1].chars().count();
                        let w = self.cols();
                        self.cursor.x = n % w;
                        self.offset.x = w * (n / w);

                        // Move line to the end of the previous line
                        let line = self.lines.remove(y);
                        self.lines[y - 1].push_str(&line);

                        // Move cursor up to the previous line
                        if self.cursor.y > 0 {
                            self.cursor.y -= 1;
                        } else {
                            self.offset.y -= 1;
                        }

                        self.print_screen();
                    }
                },
                '\x7f' => { // Delete
                    let y = self.offset.y + self.cursor.y;
                    let n = self.lines[y].chars().count();
                    if self.offset.x + self.cursor.x >= n { // Remove newline from line
                        if y + 1 < self.lines.len() {
                            let line = self.lines.remove(y + 1);
                            self.lines[y].push_str(&line);
                            self.print_screen();
                        }
                    } else { // Remove char from line
                        self.lines[y].remove(self.offset.x + self.cursor.x);
                        let line = self.render_line(y);
                        print!("\x1b[2K\x1b[1G{}", line);
                    }
                },
                c => {
                    if let Some(s) = self.render_char(c) {
                        let y = self.offset.y + self.cursor.y;
                        let mut row: Vec<char> = self.lines[y].chars().collect();
                        for c in s.chars() {
                            row.insert(self.offset.x + self.cursor.x, c);
                            self.cursor.x += 1;
                        }
                        self.lines[y] = row.into_iter().collect();
                        if self.cursor.x >= self.cols() {
                            self.offset.x += self.cols();
                            self.cursor.x -= self.cols();
                            self.print_screen();
                        } else {
                            let line = self.render_line(self.offset.y + self.cursor.y);
                            print!("\x1b[2K\x1b[1G{}", line);
                        }
                    }
                },
            }
            escape = false;
            csi = false;
            self.print_editing_status();
            print!("\x1b[{};{}H", self.cursor.y + 1, self.cursor.x + 1);
            print!("\x1b[?25h"); // Enable cursor
        }
        Ok(())
    }

    // Move cursor past end of line to end of line or left of the screen
    fn next_pos(&self, x: usize, y: usize) -> usize {
        let eol = self.lines[self.offset.y + y].chars().count();
        if eol <= self.offset.x + x {
            if eol <= self.offset.x {
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
