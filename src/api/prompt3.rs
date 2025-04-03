// prompt.rs
use crate::api::{console, fs, io};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use vte::{Params, Parser, Perform};
use core::sync::atomic::Ordering;
use crate::sys::keyboard::SHIFT;

pub struct Prompt {
    // Multi-line state
    lines: Vec<Vec<char>>,
    cursor_row: usize,
    cursor_col: usize,
    desired_horizontal: usize,
    offset: usize,
    pub eol: bool,
    
    // History state
    pub history: History,
    
    // Autocomplete state
    pub completion: Completion,
}

pub struct History {
    entries: Vec<String>,
    limit: usize,
    pos: Option<usize>,
}

pub struct Completion {
    completer: Box<dyn Fn(&str) -> Vec<String>>,
    entries: Vec<String>,
    pos: Option<usize>,
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            lines: vec![Vec::with_capacity(console::cols())],
            cursor_row: 0,
            cursor_col: 0,
            desired_horizontal: 0,
            offset: 0,
            eol: true,
            history: History {
                entries: Vec::new(),
                limit: 1000,
                pos: None,
            },
            completion: Completion {
                completer: Box::new(|_| Vec::new()),
                entries: Vec::new(),
                pos: None,
            },
        }
    }

    fn max_line_width(&self) -> usize {
        console::cols() - if self.cursor_row == 0 { self.offset } else { 1 }
    }

    fn current_line_index(&self) -> usize {
        self.cursor_col.saturating_sub(
            if self.cursor_row == 0 { self.offset } else { 0 }
        )
    }

    fn current_line(&self) -> &Vec<char> {
        &self.lines[self.cursor_row]
    }

    fn current_line_mut(&mut self) -> &mut Vec<char> {
        &mut self.lines[self.cursor_row]
    }

    pub fn input(&mut self, prompt: &str) -> Option<String> {
        print!("{}", prompt);
        self.offset = offset_from_prompt(prompt);
        self.cursor_row = 0;
        self.cursor_col = self.offset;
        self.desired_horizontal = 0;
        self.lines = vec![Vec::with_capacity(console::cols())];
        
        let mut parser = Parser::new();
        while let Some(c) = io::stdin().read_char() {
            match c {
                console::ETX_KEY => {
                    if self.eol {
                        println!();
                    }
                    return Some(String::new());
                },
                console::EOT_KEY => return self.handle_ctrl_d(),
                '\n' => return self.handle_enter(),
                '\t' => self.handle_tab(),
                _ => self.process_char(c, &mut parser),
            }
        }
        None
    }

    // fn handle_ctrl_c(&mut self) -> Option<String> {
    //     if self.eol { println!(); }
    //     Some(String::new())
    // }

    fn handle_ctrl_d(&mut self) -> Option<String> {
        if self.current_line().is_empty() {
            if self.eol { 
                println!(); 
            }
            let result = self.collect_input();
            if !result.is_empty() {
                self.history.add(result.clone());
            }
            Some(result)
        } else {
            self.handle_delete();
            None
        }
    }

    fn handle_enter(&mut self) -> Option<String> {
        if SHIFT.load(Ordering::Relaxed) {
            self.insert_newline();
            None
        } else {
            if self.eol { println!(); }
            let result = self.collect_input();
            if !result.is_empty() {
                self.history.add(result.clone());
            }
            Some(result)
        }
    }

    fn insert_newline(&mut self) {
        // Primeiro obtemos o índice de forma imutável
        let split_pos = self.current_line_index();
        
        // Depois obtemos a referência mutável
        let current_line = self.current_line_mut();
        let new_line = current_line.split_off(split_pos);
        
        // Restante das operações
        self.lines.insert(self.cursor_row + 1, new_line);
        self.cursor_row += 1;
        self.cursor_col = 0;
        self.desired_horizontal = 0;
    }

    fn collect_input(&self) -> String {
        self.lines.iter()
            .map(|line| line.iter().collect())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn handle_tab(&mut self) {
        let line: String = self.current_line().iter().collect();
        self.completion.entries = (self.completion.completer)(&line);
        
        // Generate completions outside of borrow
        let entries = (self.completion.completer)(&line);
        
        // Store completion in temporary variable
        let completion_str = entries.first().cloned();
        
        // Update completion state
        self.completion.entries = entries;
        
        if let Some(completion) = completion_str {
            self.insert_completion(&completion);
            self.completion.pos = Some(0);
        }
    }

    fn insert_completion(&mut self, completion: &str) {
        let insert_pos = self.current_line_index();
        let line = self.current_line_mut();
        for (i, c) in completion.chars().enumerate() {
            line.insert(insert_pos + i, c);
        }
        
        self.cursor_col += completion.len();
        self.desired_horizontal = self.current_line_index();
    }

    fn process_char(&mut self, c: char, parser: &mut Parser) {
        if console::is_printable(c) {
            self.insert_char(c);
        }
        for b in c.to_string().as_bytes() {
            parser.advance(self, *b);
        }
        self.completion.entries.clear();
    }

    fn handle_printable_key(&mut self, c: char) {
        if console::is_printable(c) {
            let idx = self.current_line_index();
            let max = self.max_line_width();
            let current_line = self.current_line_mut();
            
            // Handle line wrapping
            if current_line.len() >= max {
                self.handle_line_overflow(c);
            } else {
                current_line.insert(idx, c);
                self.cursor_col += 1;
                self.desired_horizontal = idx + 1;
            }
        }
    }

    fn handle_line_overflow(&mut self, c: char) {
        let idx = self.current_line_index();
        // Split current line at cursor position
        let new_line = self.current_line_mut().split_off(idx);
        
        // Create new line with the overflow character
        self.lines.insert(self.cursor_row + 1, vec![c]);
        
        // Add the split content after the new character
        self.lines[self.cursor_row + 1].extend(new_line);
        
        // Move cursor to new line
        self.cursor_row += 1;
        self.cursor_col = 1;  // After the new character
        self.desired_horizontal = 0;
    }

    fn move_cursor_right(&mut self) {
        let current_line_len = self.current_line().len();
        let max_col = self.offset + current_line_len;
        
        if self.cursor_col < max_col {
            // Move within current line
            self.cursor_col += 1;
            self.desired_horizontal += 1;
        } else if self.cursor_row < self.lines.len() - 1 {
            // Move to next line
            self.cursor_row += 1;
            self.cursor_col = 0;
            self.desired_horizontal = 0;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_col > self.offset {
            // Move within current line
            self.cursor_col -= 1;
            self.desired_horizontal = self.desired_horizontal.saturating_sub(1);
        } else if self.cursor_row > 0 {
            // Move to previous line
            self.cursor_row -= 1;
            let prev_line_len = self.current_line().len();
            self.cursor_col = self.offset + prev_line_len;
            self.desired_horizontal = prev_line_len;
        }
    }

    fn insert_char(&mut self, c: char) {
        let pos = self.current_line_index();
        let max = self.max_line_width();
        let line = self.current_line_mut();
        
        if line.len() < max {
            line.insert(pos, c);
            self.cursor_col += 1;
            self.desired_horizontal = pos + 1;
        } else if self.cursor_row == self.lines.len() - 1 {
            self.lines.push(vec![c]);
            self.cursor_row += 1;
            self.cursor_col = 1;
            self.desired_horizontal = 0;
        }
    }

    // fn update_display(&self) {
    //     let line = self.current_line();
    //     let line_str: String = line.iter().collect();
    //     let prompt = if self.cursor_row == 0 { 
    //         "> ".to_string() 
    //     } else { 
    //         "  ".to_string() 
    //     };
        
    //     print!("\r\x1b[K{}{}", prompt, line_str);
    //     self.set_cursor_pos();
    // }

    // fn set_cursor_pos(&self) {
    //     let col = self.cursor_col - if self.cursor_row == 0 { self.offset } else { 0 };
    //     print!("\x1b[{};{}H", self.cursor_row + 1, col + 1);
    // }

    fn handle_backspace(&mut self) {
        if self.current_line_index() > 0 {
            let pos = self.current_line_index() - 1;
            let line = self.current_line_mut();
            line.remove(pos);
            self.cursor_col -= 1;
            self.desired_horizontal = pos;
        } else if self.cursor_row > 0 {
            let current_line = self.lines.remove(self.cursor_row);
            self.cursor_row -= 1;
            let prev_len = self.current_line().len();
            self.current_line_mut().extend(current_line);
            self.cursor_col = prev_len + if self.cursor_row == 0 { self.offset } else { 0 };
            self.desired_horizontal = prev_len;
        }
    }

    fn handle_delete(&mut self) {
        if self.current_line_index() < self.current_line().len() {
            let idx = self.current_line_index();
            let line = self.current_line_mut();
            line.remove(idx);
        } else if self.cursor_row < self.lines.len() - 1 {
            let next_line = self.lines.remove(self.cursor_row + 1);
            self.current_line_mut().extend(next_line);
        }
    }

    fn navigate_history(&mut self, up: bool) {
        let current = self.collect_input();
        if self.history.pos.is_none() && !current.is_empty() {
            self.history.entries.push(current);
        }

        let new_pos = match (up, self.history.pos) {
            (true, Some(pos)) if pos > 0 => Some(pos - 1),
            (false, Some(pos)) => Some(pos + 1),
            (true, None) if !self.history.entries.is_empty() => {
                Some(self.history.entries.len() - 1)
            }
            _ => None,
        };

        if let Some(pos) = new_pos {
            if pos < self.history.entries.len() {
                self.load_history_entry(pos);
                return;
            }
        }
        
        if !up {
            self.restore_current_input();
        }
    }

    fn load_history_entry(&mut self, pos: usize) {
        let entry = self.history.entries[pos].clone();
        self.lines = entry.split('\n')
            .map(|line| line.chars().collect())
            .collect();
        self.cursor_row = self.lines.len().saturating_sub(1);
        self.cursor_col = self.lines[self.cursor_row].len();
        self.history.pos = Some(pos);
    }

    fn restore_current_input(&mut self) {
        self.history.pos = None;
        // Implement current input restoration if needed
    }
}

impl Perform for Prompt {
    fn execute(&mut self, b: u8) {
        match b as char {
            '\x08' => self.handle_backspace(),
            _ => {}
        }
    }

    fn print(&mut self, c: char) {
        match c {
            '\x7f' => self.handle_delete(),
            c => self.handle_printable_key(c),
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _: &[u8], _: bool, c: char) {
        match c {
            'A' => self.navigate_history(true),   // Up arrow
            'B' => self.navigate_history(false),  // Down arrow
            'C' => self.move_cursor_right(),
            'D' => self.move_cursor_left(),
            '~' => {
                for param in params.iter() {
                    match param[0] {
                        3 => self.handle_delete(),
                        5 => print!("\x1b[5~"),  // Page Up
                        6 => print!("\x1b[6~"),  // Page Down
                        _ => continue,
                    }
                }
            }
            _ => {}
        }
    }
}

impl History {
    pub fn add(&mut self, entry: String) {
        self.entries.retain(|e| e != &entry);
        self.entries.push(entry);
        while self.entries.len() > self.limit {
            self.entries.remove(0);
        }
    }

    pub fn load(&mut self, path: &str) {
        if let Ok(contents) = fs::read_to_string(path) {
            self.entries = contents.lines()
                .map(|s| s.to_string())
                .collect();
        }
    }

    pub fn save(&self, path: &str) {
        let _ = fs::write(path, self.entries.join("\n").as_bytes()).ok();
    }
}

impl Completion {
    pub fn set(&mut self, completer: &'static dyn Fn(&str) -> Vec<String>) {
        self.completer = Box::new(completer);
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