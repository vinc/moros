use crate::api::fs;
use crate::api::console;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use vte::{Params, Parser, Perform};

pub struct Prompt {
    pub history: History,
    offset: usize, // Offset cursor position by prompt length
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            history: History::new(),
            offset: 0,
        }
    }

    pub fn input(&mut self, prompt: &str) -> Option<String> {
        print!("{}", prompt);
        self.offset = offset_from_prompt(prompt);
        let mut cursor = self.offset;
        let mut line = String::new();
        let mut escape_sequence = false;
        let mut control_sequence = false;
        while let Some(c) = console::read_char() {
            match c {
                '\x03' => { // End of Text (^C)
                    print!("\n");
                    return None;
                },
                '\x04' => { // End of Transmission (^D)
                    print!("\n");
                    return None;
                },
                '\x1b' => {
                    escape_sequence = true;
                    continue;
                },
                '[' if escape_sequence => {
                    control_sequence = true;
                    continue;
                },
                'A' if control_sequence => { // Cursor Up
                    // TODO: Navigate history up
                },
                'B' if control_sequence => { // Cursor Down
                    // TODO: Navigate history down
                },
                'C' if control_sequence => { // Cursor Forward
                    if cursor < self.offset + line.len() {
                        print!("\x1b[1C");
                        cursor += 1;
                    }
                },
                'D' if control_sequence => { // Cursor Backward
                    if cursor > self.offset {
                        print!("\x1b[1D");
                        cursor -= 1;
                    }
                },
                '\n' => { // New Line
                    print!("{}", c);
                    return Some(line);
                },
                '\x08' => { // Backspace
                    if cursor > self.offset {
                        let i = cursor - self.offset - 1;
                        line.remove(i);
                        let s = &line[i..];
                        print!("{}{} \x1b[{}D", c, s, s.len() + 1);
                        cursor -= 1;
                    }
                },
                '\x7f' => { // Delete
                    if cursor < self.offset + line.len() {
                        let i = cursor - self.offset;
                        line.remove(i);
                        let s = &line[i..];
                        print!("{} \x1b[{}D", s, s.len() + 1);
                    }
                },
                _ => {
                    if console::is_printable(c) {
                        let i = cursor - self.offset;
                        line.insert(i, c);
                        let s = &line[i..];
                        print!("{} \x1b[{}D", s, s.len());
                        cursor += 1;
                    }
                },
            } 
            escape_sequence = false;
            control_sequence = false;
        }

        None
    }
}

pub struct History {
    entries: Vec<String>,
    limit: usize,
}

impl History {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            limit: 1000,
        }
    }

    pub fn load(&mut self, path: &str) {
        if let Ok(lines) = fs::read_to_string(path) {
            self.entries = lines.split("\n").map(|s| s.to_string()).collect();
        }
    }

    pub fn save(&mut self, path: &str) {
        fs::write(path, self.entries.join("\n").as_bytes()).ok();
    }

    pub fn add(&mut self, entry: &str) {
        // Remove duplicated entries
        let mut i = 0;
        while i < self.entries.len() {
            if self.entries[i] == entry {
                self.entries.remove(i);
            } else {
                i += 1;
            }
        }

        self.entries.push(entry.to_string());

        // Remove oldest entries if limit is reached
        while self.entries.len() > self.limit {
            self.entries.remove(0);
        }
    }
}

struct Offset(usize);

impl Perform for Offset {
    fn print(&mut self, c: char) {
        self.0 += c.len_utf8();
    }
}

fn offset_from_prompt(s: &str) -> usize {
    let mut parser = Parser::new();
    let mut offset = Offset(0);

    for b in s.bytes() {
        parser.advance(&mut offset, b);
    }
    offset.0
}
