use crate::api::fs;
use crate::api::console;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use vte::{Params, Parser, Perform};

pub struct Prompt {
    pub history: History,
    offset: usize, // Offset line by the length of the prompt string
    cursor: usize,
    line: String,
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            history: History::new(),
            offset: 0,
            cursor: 0,
            line: String::new(),
        }
    }

    pub fn input(&mut self, prompt: &str) -> Option<String> {
        print!("{}", prompt);
        self.offset = offset_from_prompt(prompt);
        self.cursor = self.offset;
        self.line = String::new();
        let mut parser = Parser::new();
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
                '\n' => { // New Line
                    print!("{}", c);
                    return Some(self.line.clone());
                },
                c => {
                    if c.is_ascii() {
                        parser.advance(self, c as u8);
                    }
                }
            }
        }

        None
    }

    fn handle_up_key(&mut self) {
        // TODO: Navigate history up
    }

    fn handle_down_key(&mut self) {
        // TODO: Navigate history down
    }

    fn handle_forward_key(&mut self) {
        if self.cursor < self.offset + self.line.len() {
            print!("\x1b[1C");
            self.cursor += 1;
        }
    }

    fn handle_backward_key(&mut self) {
        if self.cursor > self.offset {
            print!("\x1b[1D");
            self.cursor -= 1;
        }
    }

    fn handle_delete_key(&mut self) {
        if self.cursor < self.offset + self.line.len() {
            let i = self.cursor - self.offset;
            self.line.remove(i);
            let s = &self.line[i..];
            print!("{} \x1b[{}D", s, s.len() + 1);
        }
    }

    fn handle_backspace_key(&mut self) {
        if self.cursor > self.offset {
            let i = self.cursor - self.offset - 1;
            self.line.remove(i);
            let s = &self.line[i..];
            print!("{}{} \x1b[{}D", '\x08', s, s.len() + 1);
            self.cursor -= 1;
        }
    }

    fn handle_printable_key(&mut self, c: char) {
        if console::is_printable(c) {
            let i = self.cursor - self.offset;
            self.line.insert(i, c);
            let s = &self.line[i..];
            print!("{} \x1b[{}D", s, s.len());
            self.cursor += 1;
        }
    }
}

impl Perform for Prompt {
    fn execute(&mut self, b: u8) {
        let c = b as char;
        match c {
            '\x08' => self.handle_backspace_key(),
            _ => {},
        }
    }

    fn print(&mut self, c: char) {
        match c {
            '\x7f' => self.handle_delete_key(),
            c => self.handle_printable_key(c),
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'A' => self.handle_up_key(),
            'B' => self.handle_down_key(),
            'C' => self.handle_forward_key(),
            'D' => self.handle_backward_key(),
            '~' => {
                for param in params.iter() {
                    if param[0] == 3 { // Delete
                        self.handle_delete_key();
                    }
                }
            },
            _ => {},
        }
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
