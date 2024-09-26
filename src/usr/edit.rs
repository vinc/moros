use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::prompt::Prompt;
use crate::api::regex::Regex;
use crate::api::{console, fs, io};
use crate::api;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp;

enum Cmd {
    Save,
    Replace,
    Delete,
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
    highlighted: Vec<(usize, usize, char)>,
    config: EditorConfig,
    search_prompt: Prompt,
    search_query: String,
    command_prompt: Prompt,
    command_history: String,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        let cursor = Coords { x: 0, y: 0 };
        let offset = Coords { x: 0, y: 0 };
        let highlighted = Vec::new();
        let clipboard = Vec::new();
        let mut lines = Vec::new();
        let config = EditorConfig { tab_size: 4 };

        let search_query = String::new();
        let mut search_prompt = Prompt::new();
        search_prompt.eol = false;

        let mut command_prompt = Prompt::new();
        let command_history = String::from("~/.edit-history");
        command_prompt.history.load(&command_history);
        command_prompt.eol = false;

        match fs::read_to_string(pathname) {
            Ok(contents) => {
                for line in contents.lines() {
                    lines.push(line.into());
                }
                if lines.is_empty() {
                    lines.push(String::new());
                }
            }
            Err(_) => {
                lines.push(String::new());
            }
        };

        let pathname = pathname.into();

        Self {
            pathname,
            clipboard,
            lines,
            cursor,
            offset,
            highlighted,
            config,
            search_prompt,
            search_query,
            command_prompt,
            command_history,
        }
    }

    pub fn save(&mut self, path: &str) -> Result<(), ExitCode> {
        let contents = self.lines.join("\n") + "\n";

        if fs::write(path, contents.as_bytes()).is_ok() {
            self.pathname = path.into();
            let n = self.lines.len();
            let status = format!("Wrote {}L to '{}'", n, path);
            self.print_status(&status, "yellow");
            Ok(())
        } else {
            let status = format!("Could not write to '{}'", path);
            self.print_status(&status, "red");
            Err(ExitCode::Failure)
        }
    }

    fn print_status(&mut self, status: &str, background: &str) {
        // Move cursor to the bottom of the screen
        print!("\x1b[{};1H", rows() + 1);

        let color = Style::color("black").with_background(background);
        let reset = Style::reset();
        print!("{}{:cols$}{}", color, status, reset, cols = cols());

        // Move cursor back
        print!("\x1b[{};{}H", self.cursor.y + 1, self.cursor.x + 1);
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

        let width = cols() - start.chars().count();
        let status = format!("{}{:>width$}", start, end, width = width);

        self.print_status(&status, "silver");
    }

    fn print_screen(&mut self) {
        let mut lines: Vec<String> = Vec::new();
        let a = self.offset.y;
        let b = self.offset.y + rows();
        for y in a..b {
            lines.push(self.render_line(y));
        }
        println!("\x1b[1;1H{}", lines.join("\n"));
    }

    fn render_line(&self, y: usize) -> String {
        // Render line into a row of the screen, or an empty row when past EOF
        let line = if y < self.lines.len() {
            &self.lines[y]
        } else {
            ""
        };

        let s = format!("{:cols$}", line, cols = self.offset.x);
        let mut row: Vec<char> = s.chars().collect();
        let n = self.offset.x + cols();
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
            '\t' => Some(" ".repeat(self.config.tab_size)),
            c if console::is_printable(c) => Some(c.to_string()),
            _ => None,
        }
    }

    fn match_chars(&mut self, opening: char, closing: char) {
        let mut stack = Vec::new();
        let ox = self.offset.x;
        let oy = self.offset.y;
        let cx = self.cursor.x;
        let cy = self.cursor.y;
        if let Some(cursor) = self.lines[oy + cy].chars().nth(ox + cx) {
            if cursor == closing {
                for (y, line) in self.lines.iter().enumerate() {
                    for (x, c) in line.chars().enumerate() {
                        if oy + cy == y && ox + cx == x {
                            // Cursor position
                            if let Some((x, y)) = stack.pop() {
                                self.highlighted.push((cx, cy, closing));
                                let is_col = ox <= x && x < ox + cols();
                                let is_row = oy <= y && y < oy + rows();
                                if is_col && is_row {
                                    self.highlighted.push(
                                        (x - ox, y - oy, opening)
                                    );
                                }
                            }
                            return;
                        }
                        if c == opening {
                            stack.push((x, y));
                        }
                        if c == closing {
                            stack.pop();
                        }
                    }
                    if oy + cy == y {
                        break;
                    }
                }
            }
            if cursor == opening {
                for (y, line) in self.lines.iter().enumerate().skip(oy + cy) {
                    for (x, c) in line.chars().enumerate() {
                        if y == oy + cy && x <= ox + cx {
                            continue; // Skip chars before cursor
                        }
                        if c == opening {
                            stack.push((x, y));
                        }
                        if c == closing {
                            if stack.pop().is_none() {
                                self.highlighted.push((cx, cy, opening));
                                let is_col = ox <= x && x < ox + cols();
                                let is_row = oy <= y && y < oy + rows();
                                if is_col && is_row {
                                    self.highlighted.push(
                                        (x - ox, y - oy, closing)
                                    );
                                }
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    fn print_highlighted(&mut self) {
        self.match_chars('(', ')');
        self.match_chars('{', '}');
        self.match_chars('[', ']');
        let color = Style::color("red");
        let reset = Style::reset();
        for (x, y, c) in &self.highlighted {
            if *x == cols() - 1 {
                continue;
            }
            print!("\x1b[{};{}H", y + 1, x + 1);
            print!("{}{}{}", color, c, reset);
        }
    }

    fn clear_highlighted(&mut self) {
        let reset = Style::reset();
        for (x, y, c) in &self.highlighted {
            if *x == cols() - 1 {
                continue;
            }
            print!("\x1b[{};{}H", y + 1, x + 1);
            print!("{}{}", reset, c);
        }
        self.highlighted.clear();
    }

    // Align cursor that is past the end of the line, to the end
    // of the line.
    //
    // If the cursor is somewhere on the long line on the second
    // screen in the following diagram, going down should move
    // the cursor to the end of the short line and display the
    // first screen instead of the second screen.
    //
    // +----------------------------+----------------------------+
    // |                            |                            |
    // | This is a loooooooooooooooo|oooooong line               |
    // | This is a short line       |          ^                 |
    // |                     ^      |                            |
    // +----------------------------+----------------------------+
    fn align_cursor(&mut self) {
        let x = self.offset.x + self.cursor.x;
        let y = self.offset.y + self.cursor.y;
        let eol = self.lines[y].chars().count();
        if x > eol {
            let n = cols();
            self.offset.x = (eol / n) * n;
            self.cursor.x = eol % n;
        }
    }

    pub fn run(&mut self) -> Result<(), ExitCode> {
        print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
        self.print_screen();
        self.print_editing_status();
        self.print_highlighted();
        print!("\x1b[1;1H"); // Move cursor to the top of the screen

        let mut escape = false;
        let mut csi = false;
        let mut csi_params = String::new();
        loop {
            let c = io::stdin().read_char().unwrap_or('\0');
            print!("\x1b[?25l"); // Disable cursor
            self.clear_highlighted();
            print!("\x1b[{};{}H", self.cursor.y + 1, self.cursor.x + 1);

            match c {
                '\x1B' => { // ESC
                    escape = true;
                    continue;
                }
                '[' if escape => {
                    csi = true;
                    csi_params.clear();
                    continue;
                }
                '\0' => {
                    continue;
                }
                '\x11' | '\x03' => { // Ctrl Q or Ctrl C
                    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
                    print!("\x1b[?25h"); // Enable cursor
                    break;
                }
                '\x17' => { // Ctrl W
                    self.save(&self.pathname.clone()).ok();
                    print!("\x1b[?25h"); // Enable cursor
                    continue;
                }
                '\x18' => { // Ctrl X
                    let res = self.save(&self.pathname.clone());
                    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
                    print!("\x1b[?25h"); // Enable cursor
                    return res;
                }
                '\n' => { // Newline
                    let y = self.offset.y + self.cursor.y;
                    let old_line = self.lines[y].clone();
                    let mut row: Vec<char> = old_line.chars().collect();
                    let new_line = row.
                        split_off(self.offset.x + self.cursor.x).
                        into_iter().collect();
                    self.lines[y] = row.into_iter().collect();
                    self.lines.insert(y + 1, new_line);
                    if self.cursor.y == rows() - 1 {
                        self.offset.y += 1;
                    } else {
                        self.cursor.y += 1;
                    }
                    self.cursor.x = 0;
                    self.offset.x = 0;
                    self.print_screen();
                }
                '~' if csi && csi_params == "5" => { // Page Up
                    let scroll = rows() - 1; // Keep one line on screen
                    self.offset.y -= cmp::min(scroll, self.offset.y);
                    self.print_screen();
                }
                '~' if csi && csi_params == "6" => { // Page Down
                    let scroll = rows() - 1; // Keep one line on screen
                    let n = cmp::max(self.lines.len(), 1);
                    let remaining = n - self.offset.y - 1;
                    self.offset.y += cmp::min(scroll, remaining);
                    if self.cursor.y + scroll > remaining {
                        self.cursor.y = 0;
                    }
                    self.print_screen();
                }
                'A' if csi => { // Arrow Up
                    if self.cursor.y > 0 {
                        self.cursor.y -= 1
                    } else if self.offset.y > 0 {
                        self.offset.y -= 1;
                    }
                    self.align_cursor();
                    self.print_screen();
                }
                'B' if csi => { // Arrow Down
                    let n = self.lines.len() - 1;
                    let is_eof = n == (self.offset.y + self.cursor.y);
                    let is_bottom = self.cursor.y == rows() - 1;
                    if self.cursor.y < cmp::min(rows(), n) {
                        if is_bottom || is_eof {
                            if !is_eof {
                                self.offset.y += 1;
                            }
                        } else {
                            self.cursor.y += 1;
                        }
                        self.align_cursor();
                        self.print_screen();
                    }
                }
                'C' if csi => { // Arrow Right
                    let line = &self.lines[self.offset.y + self.cursor.y];
                    let x = self.cursor.x + self.offset.x;
                    let n = line.chars().count();
                    if line.is_empty() || x >= n {
                        print!("\x1b[?25h"); // Enable cursor
                        escape = false;
                        csi = false;
                        continue;
                    } else if self.cursor.x == cols() - 1 {
                        self.offset.x += cols();
                        self.cursor.x -= cols() - 1;
                        self.print_screen();
                    } else {
                        self.cursor.x += 1;
                    }
                }
                'D' if csi => { // Arrow Left
                    if self.cursor.x + self.offset.x == 0 {
                        print!("\x1b[?25h"); // Enable cursor
                        escape = false;
                        csi = false;
                        continue;
                    } else if self.cursor.x == 0 {
                        self.offset.x -= cols();
                        self.cursor.x += cols() - 1;
                        self.align_cursor();
                        self.print_screen();
                    } else {
                        self.cursor.x -= 1;
                    }
                }
                'Z' if csi => { // Backtab (Shift + Tab)
                     // Do nothing
                }
                '\x14' => { // Ctrl T -> Go to top of file
                    self.cursor.x = 0;
                    self.cursor.y = 0;
                    self.offset.x = 0;
                    self.offset.y = 0;
                    self.print_screen();
                }
                '\x02' => { // Ctrl B -> Go to bottom of file
                    self.cursor.x = 0;
                    self.cursor.y = cmp::min(rows(), self.lines.len()) - 1;
                    self.offset.x = 0;
                    self.offset.y = self.lines.len() - 1 - self.cursor.y;
                    self.print_screen();
                }
                '\x01' => { // Ctrl A -> Go to beginning of line
                    self.cursor.x = 0;
                    self.offset.x = 0;
                    self.print_screen();
                }
                '\x05' => { // Ctrl E -> Go to end of line
                    let line = &self.lines[self.offset.y + self.cursor.y];
                    let n = line.chars().count();
                    let w = cols();
                    self.cursor.x = n % w;
                    self.offset.x = w * (n / w);
                    self.print_screen();
                }
                '\x04' => { // Ctrl D -> Delete (cut) line
                    let i = self.offset.y + self.cursor.y;
                    self.clipboard.push(self.lines.remove(i));
                    if self.lines.is_empty() {
                        self.lines.push(String::new());
                    }

                    // Move cursor up to the previous line
                    if i >= self.lines.len() {
                        if self.cursor.y > 0 {
                            self.cursor.y -= 1;
                        } else if self.offset.y > 0 {
                            self.offset.y -= 1;
                        }
                    }
                    self.cursor.x = 0;
                    self.offset.x = 0;

                    self.print_screen();
                }
                '\x19' => { // Ctrl Y -> Yank (copy) line
                    let i = self.offset.y + self.cursor.y;
                    self.clipboard.push(self.lines[i].clone());
                }
                '\x10' => { // Ctrl P -> Put (paste) line
                    let i = self.offset.y + self.cursor.y;
                    if let Some(line) = self.clipboard.pop() {
                        self.lines.insert(i + 1, line);
                    }
                    self.cursor.x = 0;
                    self.offset.x = 0;
                    self.print_screen();
                }
                '\x06' => { // Ctrl F -> Find
                    self.find();
                    self.print_screen();
                }
                '\x0E' => { // Ctrl N -> Find next
                    self.find_next();
                    self.print_screen();
                }
                '\x0C' => { // Ctrl L -> Line mode
                    match self.exec() {
                        Some(Cmd::Save) => {
                            print!("\x1b[?25h"); // Enable cursor
                            continue;
                        }
                        Some(_) => {
                            self.print_screen();
                        }
                        None => {
                        }
                    }
                }
                '\x08' => { // Backspace
                    let y = self.offset.y + self.cursor.y;
                    if self.offset.x + self.cursor.x > 0 {
                        // Remove char from line
                        let mut row: Vec<_> = self.lines[y].chars().collect();
                        row.remove(self.offset.x + self.cursor.x - 1);
                        self.lines[y] = row.into_iter().collect();

                        if self.cursor.x == 0 {
                            self.offset.x -= cols();
                            self.cursor.x = cols() - 1;
                            self.print_screen();
                        } else {
                            self.cursor.x -= 1;
                            let line = self.render_line(y);
                            print!("\x1b[2K\x1b[1G{}", line);
                        }
                    } else {
                        // Remove newline from previous line
                        if self.cursor.y == 0 && self.offset.y == 0 {
                            print!("\x1b[?25h"); // Enable cursor
                            escape = false;
                            csi = false;
                            continue;
                        }

                        // Move cursor below the end of the previous line
                        let n = self.lines[y - 1].chars().count();
                        let w = cols();
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
                }
                '\x7f' => {
                    // Delete
                    let y = self.offset.y + self.cursor.y;
                    let n = self.lines[y].chars().count();
                    if self.offset.x + self.cursor.x >= n {
                        // Remove newline from line
                        if y + 1 < self.lines.len() {
                            let line = self.lines.remove(y + 1);
                            self.lines[y].push_str(&line);
                            self.print_screen();
                        }
                    } else {
                        // Remove char from line
                        self.lines[y].remove(self.offset.x + self.cursor.x);
                        let line = self.render_line(y);
                        print!("\x1b[2K\x1b[1G{}", line);
                    }
                }
                c if csi => {
                    csi_params.push(c);
                    continue;
                }
                c => {
                    if let Some(s) = self.render_char(c) {
                        let y = self.offset.y + self.cursor.y;
                        let mut row: Vec<_> = self.lines[y].chars().collect();
                        for c in s.chars() {
                            row.insert(self.offset.x + self.cursor.x, c);
                            self.cursor.x += 1;
                        }
                        self.lines[y] = row.into_iter().collect();
                        if self.cursor.x >= cols() {
                            self.offset.x += cols();
                            self.cursor.x -= cols();
                            self.print_screen();
                        } else {
                            let line = self.render_line(y);
                            print!("\x1b[2K\x1b[1G{}", line);
                        }
                    }
                }
            }
            self.print_editing_status();
            self.print_highlighted();
            print!("\x1b[{};{}H", self.cursor.y + 1, self.cursor.x + 1);
            print!("\x1b[?25h"); // Enable cursor
            escape = false;
            csi = false;
        }
        Ok(())
    }

    fn exec(&mut self) -> Option<Cmd> {
        if let Some(cmd) = prompt(&mut self.command_prompt, ":") {
            // The cursor is disabled at the beginning of the loop in the `run`
            // method to avoid seeing it jump around during screen operations.
            // The `prompt` method above re-enable the cursor so we need to
            // disable it again until the end of the loop in the `run` method.
            print!("\x1b[?25l");

            self.exec_command(&cmd)
        } else {
            None
        }
    }

    fn exec_command(&mut self, cmd: &str) -> Option<Cmd> {
        let mut res = None;
        let params: Vec<&str> = match cmd.chars().next() {
            Some('w') =>  {
                cmd.split(' ').collect()
            }
            _ => {
                cmd.split('/').collect()
            }
        };
        // TODO: Display line numbers on screen and support command range
        match params[0] {
            "d" if params.len() == 1 => { // Delete current line
                let y = self.offset.y + self.cursor.y;
                self.lines.remove(y);
                res = Some(Cmd::Delete);
            }
            "%d" if params.len() == 1 => { // Delete all lines
                self.lines = vec![String::new()];
                res = Some(Cmd::Delete);
            }
            "g" if params.len() == 3 => { // Global command
                let re = Regex::new(params[1]);
                if params[2] == "d" { // Delete all matching lines
                    self.lines.retain(|line| !re.is_match(line));
                    res = Some(Cmd::Delete);
                }
            }
            "s" if params.len() == 4 => { // Substitute current line
                let re = Regex::new(params[1]);
                let s = params[2];
                let y = self.offset.y + self.cursor.y;
                if params[3] == "g" { // Substitute all occurrences
                    self.lines[y] = re.replace_all(&self.lines[y], s);
                } else {
                    self.lines[y] = re.replace(&self.lines[y], s);
                }
                res = Some(Cmd::Replace);
            }
            "%s" if params.len() == 4 => { // Substitute all lines
                let re = Regex::new(params[1]);
                let s = params[2];
                let n = self.lines.len();
                for y in 0..n {
                    if params[3] == "g" { // Substitute all occurrences
                        self.lines[y] = re.replace_all(&self.lines[y], s);
                    } else {
                        self.lines[y] = re.replace(&self.lines[y], s);
                    }
                }
                res = Some(Cmd::Replace);
            }
            "w" => { // Save file
                let path = if params.len() == 2 {
                    params[1]
                } else {
                    &self.pathname.clone()
                };
                self.save(path).ok();
                res = Some(Cmd::Save);
            }
            _ => {}
        }

        if res.is_some() {
            let mut y = self.offset.y + self.cursor.y;
            let n = self.lines.len() - 1;
            if y > n {
                self.cursor.y = n % rows();
                self.offset.y = n - self.cursor.y;
                y = n;
            }
            let n = self.lines[y].len();
            if self.offset.x + self.cursor.x > n {
                self.cursor.x = n % cols();
                self.offset.x = n - self.cursor.x;
            }

            self.command_prompt.history.add(cmd);
            self.command_prompt.history.save(&self.command_history);
        }

        res
    }

    pub fn find(&mut self) {
        if let Some(query) = prompt(&mut self.search_prompt, "Find: ") {
            if !query.is_empty() {
                self.search_prompt.history.add(&query);
                self.search_query = query;
                self.find_next();
            }
        }
    }

    pub fn find_next(&mut self) {
        let dx = self.offset.x + self.cursor.x;
        let dy = self.offset.y + self.cursor.y;
        for (y, line) in self.lines.iter().enumerate() {
            let mut o = 0;
            if y < dy {
                continue;
            }
            if y == dy {
                o = cmp::min(dx + 1, line.len());
            }
            if let Some(i) = line[o..].find(&self.search_query) {
                let x = o + i;
                self.cursor.x = x % cols();
                self.cursor.y = y % rows();
                self.offset.x = x - self.cursor.x;
                self.offset.y = y - self.cursor.y;
                break;
            }
        }
    }
}

pub fn prompt(prompt: &mut Prompt, label: &str) -> Option<String> {
    let color = Style::color("black").with_background("silver");
    let reset = Style::reset();

    // Set up the bottom line for the prompt
    print!("\x1b[{};1H", rows() + 1);
    print!("{}{}", color, " ".repeat(cols()));
    print!("\x1b[{};1H", rows() + 1);
    print!("\x1b[?25h"); // Enable cursor

    let res = prompt.input(label);
    print!("{}", reset);
    res
}

pub fn rows() -> usize {
    api::console::rows() - 1 // Leave out one line for status line
}

pub fn cols() -> usize {
    api::console::cols()
}

fn truncated_line_indicator() -> String {
    let color = Style::color("black").with_background("silver");
    let reset = Style::reset();
    format!("{}>{}", color, reset)
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} edit {}<options> <file>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-c{1}, {0}--command <cmd>{1}    Execute command",
        csi_option, csi_reset
    );
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut path = "";
    let mut cmd = "";
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            "-c" | "--command" => {
                if i + 1 < n {
                    i += 1;
                    cmd = args[i];
                } else {
                    error!("Missing command");
                    return Err(ExitCode::UsageError);
                }
            }
            _ => {
                if args[i].starts_with('-') {
                    error!("Invalid option '{}'", args[i]);
                    return Err(ExitCode::UsageError);
                } else if path.is_empty() {
                    path = args[i];
                } else {
                    error!("Too many arguments");
                    return Err(ExitCode::UsageError);
                }
            }
        }
        i += 1;
    }
    if path.is_empty() {
        help();
        return Err(ExitCode::UsageError);
    }

    let mut editor = Editor::new(path);

    if !cmd.is_empty() {
        editor.exec_command(cmd);
        for line in editor.lines {
            println!("{}", line);
        }
        return Ok(());
    }

    editor.run()
}
