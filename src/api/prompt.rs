use crate::api::{console, fs, io};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp;
use vte::{Params, Parser, Perform};

pub struct Prompt {
    pub completion: Completion,
    pub history: History,
    offset: usize, // Offset line by the length of the prompt string
    cursor: usize,
    line: Vec<char>, // UTF-32
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            completion: Completion::new(),
            history: History::new(),
            offset: 0,
            cursor: 0,
            line: Vec::with_capacity(80),
        }
    }

    pub fn input(&mut self, prompt: &str) -> Option<String> {
        print!("{}", prompt);
        self.offset = offset_from_prompt(prompt);
        self.cursor = self.offset;
        self.line = Vec::with_capacity(80);
        let mut parser = Parser::new();
        while let Some(c) = io::stdin().read_char() {
            match c {
                console::ETX_KEY => { // End of Text (^C)
                    self.update_completion();
                    println!();
                    return Some(String::new());
                },
                console::EOT_KEY => { // End of Transmission (^D)
                    self.update_completion();
                    println!();
                    return None;
                },
                '\n' => { // New Line
                    self.update_completion();
                    self.update_history();
                    println!();
                    return Some(self.line.iter().collect());
                },
                c => {
                   for b in c.to_string().as_bytes() {
                        parser.advance(self, *b);
                    }
                }
            }
        }

        None
    }

    fn update_history(&mut self) {
        if let Some(i) = self.history.pos {
            self.line = self.history.entries[i].chars().collect();
            self.history.pos = None;
        }
    }

    fn update_completion(&mut self) {
        if let Some(i) = self.completion.pos {
            let complete = self.completion.entries[i].chars();
            self.cursor += complete.clone().count();
            self.line.extend(complete);
            self.completion.pos = None;
            self.completion.entries = Vec::new();
        }
    }

    fn handle_tab_key(&mut self) {
        self.update_history();
        let (bs, pos) = match self.completion.pos {
            Some(pos) => {
                let n = self.completion.entries.len();
                if n == 1 {
                    self.update_completion();
                    return;
                }
                let bs = self.completion.entries[pos].chars().count();
                if pos + 1 < n {
                    (bs, pos + 1)
                } else {
                    (bs, 0)
                }
            },
            None => {
                let line: String = self.line.iter().collect();
                self.completion.entries = (self.completion.completer)(&line);
                if !self.completion.entries.is_empty() {
                    (0, 0)
                } else {
                    return
                }
            },
        };
        let erase = "\x08".repeat(bs);
        let complete = &self.completion.entries[pos];
        print!("{}{}", erase, complete);
        self.completion.pos = Some(pos);
    }

    fn handle_backtab_key(&mut self) {
        self.update_history();
        let (bs, pos) = match self.completion.pos {
            Some(pos) => {
                let n = self.completion.entries.len();
                if n == 1 {
                    self.update_completion();
                    return;
                }
                let bs = self.completion.entries[pos].chars().count();
                if pos == 0 {
                    (bs, n - 1)
                } else {
                    (bs, pos - 1)
                }
            },
            None => {
                let line: String = self.line.iter().collect();
                self.completion.entries = (self.completion.completer)(&line);
                if !self.completion.entries.is_empty() {
                    (0, 0)
                } else {
                    return
                }
            },
        };
        let erase = "\x08".repeat(bs);
        let complete = &self.completion.entries[pos];
        print!("{}{}", erase, complete);
        self.completion.pos = Some(pos);
    }

    fn handle_up_key(&mut self) {
        self.update_completion();
        let n = self.history.entries.len();
        if n == 0 {
            return;
        }
        let (bs, i) = match self.history.pos {
            Some(i) => (self.history.entries[i].chars().count(), cmp::max(i, 1) - 1),
            None => (self.line.len(), n - 1),
        };
        let line = &self.history.entries[i];
        let blank = ' '.to_string().repeat((self.offset + bs) - self.cursor);
        let erase = '\x08'.to_string().repeat(bs);
        print!("{}{}{}", blank, erase, line);
        self.cursor = self.offset + line.chars().count();
        self.history.pos = Some(i);
    }

    fn handle_down_key(&mut self) {
        self.update_completion();
        let n = self.history.entries.len();
        if n == 0 {
            return;
        }
        let (bs, i) = match self.history.pos {
            Some(i) => (self.history.entries[i].chars().count(), i + 1),
            None => return,
        };
        let (pos, line) = if i < n {
            (Some(i), self.history.entries[i].clone())
        } else {
            (None, self.line.iter().collect())
        };
        let erase = '\x08'.to_string().repeat(bs);
        print!("{}{}", erase, line);
        self.cursor = self.offset + line.chars().count();
        self.history.pos = pos;
    }

    fn handle_forward_key(&mut self) {
        self.update_completion();
        self.update_history();
        if self.cursor < self.offset + self.line.len() {
            print!("\x1b[1C");
            self.cursor += 1;
        }
    }

    fn handle_backward_key(&mut self) {
        self.update_completion();
        self.update_history();
        if self.cursor > self.offset {
            print!("\x1b[1D");
            self.cursor -= 1;
        }
    }

    fn handle_delete_key(&mut self) {
        self.update_completion();
        self.update_history();
        if self.cursor < self.offset + self.line.len() {
            let i = self.cursor - self.offset;
            self.line.remove(i);
            let s = &self.line[i..]; // UTF-32
            let n = s.len() + 1;
            let s: String = s.iter().collect(); // UTF-8
            print!("{} \x1b[{}D", s, n);
        }
    }

    fn handle_backspace_key(&mut self) {
        self.update_completion();
        self.update_history();
        if self.cursor > self.offset {
            let i = self.cursor - self.offset - 1;
            self.line.remove(i);
            let s = &self.line[i..]; // UTF-32
            let n = s.len() + 1;
            let s: String = s.iter().collect(); // UTF-8
            print!("\x08{} \x1b[{}D", s, n);
            self.cursor -= 1;
        }
    }

    fn handle_printable_key(&mut self, c: char) {
        self.update_completion();
        self.update_history();
        if console::is_printable(c) {
            let i = self.cursor - self.offset;
            self.line.insert(i, c);
            let s = &self.line[i..]; // UTF-32
            let n = s.len();
            let s: String = s.iter().collect(); // UTF-8
            print!("{} \x1b[{}D", s, n);
            self.cursor += 1;
        }
    }
}

impl Perform for Prompt {
    fn execute(&mut self, b: u8) {
        let c = b as char;
        match c {
            '\x08' => self.handle_backspace_key(),
            '\t' => self.handle_tab_key(),
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
            'Z' => self.handle_backtab_key(),
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

pub struct Completion {
    completer: Box<dyn Fn(&str) -> Vec<String>>,
    entries: Vec<String>,
    pos: Option<usize>,
}

fn empty_completer(_line: &str) -> Vec<String> {
    Vec::new()
}

impl Completion {
    pub fn new() -> Self {
        Self {
            completer: Box::new(empty_completer),
            entries: Vec::new(),
            pos: None,
        }
    }
    pub fn set(&mut self, completer: &'static dyn Fn(&str) -> Vec<String>) {
        self.completer = Box::new(completer);
    }
}

pub struct History {
    entries: Vec<String>,
    limit: usize,
    pos: Option<usize>,
}

impl History {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            limit: 1000,
            pos: None,
        }
    }

    pub fn load(&mut self, path: &str) {
        if let Ok(lines) = fs::read_to_string(path) {
            self.entries = lines.split('\n').map(|s| s.to_string()).collect();
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
