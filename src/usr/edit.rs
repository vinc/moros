use crate::api;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::prompt::Prompt;
use crate::api::regex::Regex;
use crate::api::{console, fs, io};

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp;

enum Cmd {
    Delete,
    Open,
    Quit,
    Replace,
    Save,
}

struct EditorConfig {
    tab_size: usize,
}

#[derive(Clone)]
struct Coords {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone)]
pub struct Buffer {
    pathname: String,
    lines: Vec<String>,
    cursor: Coords,
    offset: Coords,
    highlighted: Vec<(usize, usize, char)>,
}

impl From<&str> for Buffer {
    fn from(pathname: &str) -> Self {
        let p: Vec<&str> = pathname.split(':').collect();
        let pathname = p[0].to_string();
        let y = p.get(1).and_then(|s| {
            s.parse::<usize>().ok()
        }).unwrap_or(1).saturating_sub(1);
        let x = p.get(2).and_then(|s| {
            s.parse::<usize>().ok()
        }).unwrap_or(1).saturating_sub(1);

        let cursor = Coords { x: x % cols(), y: y % rows() };
        let offset = Coords { x: x - cursor.x, y: y - cursor.y };
        let highlighted = Vec::new();
        let mut lines = Vec::new();

        match fs::read_to_string(&pathname) {
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

        Self {
            pathname,
            lines,
            cursor,
            offset,
            highlighted,
        }
    }
}

impl From<&Editor> for Buffer {
    fn from(editor: &Editor) -> Self {
        Buffer {
            pathname: editor.pathname.clone(),
            lines: editor.lines.clone(),
            cursor: editor.cursor.clone(),
            offset: editor.offset.clone(),
            highlighted: editor.highlighted.clone(),
        }
    }
}

pub struct Editor {
    buffer_prompt: Prompt,
    buffers: Vec<Buffer>,
    buf: usize,

    pathname: String,
    lines: Vec<String>,
    cursor: Coords,
    offset: Coords,
    highlighted: Vec<(usize, usize, char)>,

    clipboard: Option<String>,
    config: EditorConfig,
    search_prompt: Prompt,
    search_query: String,
    command_prompt: Prompt,
    command_history: String,
}

impl Editor {
    pub fn new(pathname: &str) -> Self {
        let clipboard = None;
        let config = EditorConfig { tab_size: 4 };

        let search_query = String::new();
        let mut search_prompt = Prompt::new();
        search_prompt.eol = false;

        let mut command_prompt = Prompt::new();
        let command_history = "~/.edit-history".to_string();
        command_prompt.history.load(&command_history);
        command_prompt.eol = false;

        // TODO: Add path autocompletion
        let mut buffer_prompt = Prompt::new();
        buffer_prompt.eol = false;

        let buf = Buffer::from(pathname);

        let pathname = buf.pathname.clone();
        let lines = buf.lines.clone();
        let cursor = buf.cursor.clone();
        let offset = buf.offset.clone();
        let highlighted = buf.highlighted.clone();

        let buffers = vec![buf];
        let buf = 0;

        Self {
            buffer_prompt,
            buffers,
            buf,
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
            let status = format!("Wrote {}L to {:?}", n, path);
            self.print_status(&status, "yellow");
            Ok(())
        } else {
            let status = format!("Could not write to {:?}", path);
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
        let start = format!("Editing {:?}", path);

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
        print!("\x1b[{};{}H", self.cursor.y + 1, self.cursor.x + 1);

        let mut escape = false;
        let mut csi = false;
        let mut csi_params = String::new();
        loop {
            let c = io::stdin().read_char().unwrap_or('\0');
            print!("\x1b[?25l"); // Disable cursor
            self.clear_highlighted();
            print!("\x1b[{};{}H", self.cursor.y + 1, self.cursor.x + 1);

            match c {
                '\x1B' => { // Esc
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
                '\x11' | '\x03' => { // Ctrl + Q or Ctrl + C
                    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
                    print!("\x1b[?25h"); // Enable cursor
                    break;
                }
                '\x17' => { // Ctrl + W
                    self.save(&self.pathname.clone()).ok();
                    print!("\x1b[?25h"); // Enable cursor
                    continue;
                }
                '\x18' => { // Ctrl + X
                    let res = self.save(&self.pathname.clone());
                    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
                    print!("\x1b[?25h"); // Enable cursor
                    return res;
                }
                '\n' => { // Newline
                    self.handle_newline();
                }
                '~' if csi && csi_params == "5" => { // Page Up
                    self.handle_page_up();
                }
                '~' if csi && csi_params == "6" => { // Page Down
                    self.handle_page_down();
                }
                'A' if csi => { // Arrow Up
                    self.handle_arrow_up();
                }
                'B' if csi => { // Arrow Down
                    self.handle_arrow_down();
                }
                'C' if csi && csi_params == "1;3" => { // Alt + Arrow Right
                    self.handle_ctrl_arrow_right();
                }
                'D' if csi && csi_params == "1;3" => { // Alt + Arrow Left
                    self.handle_ctrl_arrow_left();
                }
                'C' if csi && csi_params == "1;5" => { // Ctrl + Arrow Right
                    self.handle_ctrl_arrow_right();
                }
                'D' if csi && csi_params == "1;5" => { // Ctrl + Arrow Left
                    self.handle_ctrl_arrow_left();
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
                'I' if csi && csi_params == "1;5" => { // Ctrl + Tab
                    self.next_buffer();
                    self.print_screen();
                }
                'I' if csi && csi_params == "1;6" => { // Ctrl + Shift + Tab
                    self.previous_buffer();
                    self.print_screen();
                }
                '\x14' => { // Ctrl + T -> Go to top of file
                    self.cursor.x = 0;
                    self.cursor.y = 0;
                    self.offset.x = 0;
                    self.offset.y = 0;
                    self.print_screen();
                }
                '\x02' => { // Ctrl + B -> Go to bottom of file
                    self.cursor.x = 0;
                    self.cursor.y = cmp::min(rows(), self.lines.len()) - 1;
                    self.offset.x = 0;
                    self.offset.y = self.lines.len() - 1 - self.cursor.y;
                    self.print_screen();
                }
                '\x01' => { // Ctrl + A -> Go to beginning of line
                    self.cursor.x = 0;
                    self.offset.x = 0;
                    self.print_screen();
                }
                '\x05' => { // Ctrl + E -> Go to end of line
                    let line = &self.lines[self.offset.y + self.cursor.y];
                    let n = line.chars().count();
                    let w = cols();
                    self.cursor.x = n % w;
                    self.offset.x = w * (n / w);
                    self.print_screen();
                }
                '\x04' => { // Ctrl + D -> Delete (cut) line
                    self.cut_line();
                }
                '\x19' => { // Ctrl + Y -> Yank (copy) line
                    self.copy_line();
                }
                '\x10' => { // Ctrl + P -> Put (paste) line
                    self.paste_line();
                }
                '\x06' => { // Ctrl + F -> Find
                    self.find();
                    self.print_screen();
                }
                '\x0E' => { // Ctrl + N -> Find next
                    self.find_next();
                    self.print_screen();
                }
                'N' if csi && csi_params == "1;6" => { // Ctrl + Shift + N
                    self.find_prev();
                    self.print_screen();
                }
                '\x0F' => { // Ctrl + O -> Open buffer
                    self.open();
                    self.print_screen();
                }
                '\x0B' => { // Ctrl + X -> Kill buffer
                    self.kill_buffer();
                    self.print_screen();
                }
                '\x0C' => { // Ctrl + L -> Line mode
                    match self.exec() {
                        Some(Cmd::Quit) => {
                            print!("\x1b[2J"); // Clear screen
                            print!("\x1b[1;1H"); // Move to top
                            print!("\x1b[?25h"); // Enable cursor
                            break;
                        }
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

    fn handle_newline(&mut self) {
        let x = self.offset.x + self.cursor.x;
        let y = self.offset.y + self.cursor.y;

        let old_line = self.lines[y].clone();
        let mut row: Vec<char> = old_line.chars().collect();
        let new_line = row.split_off(x).into_iter().collect();
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

    fn handle_page_up(&mut self) {
        let scroll = rows() - 1; // Keep one line on screen
        self.offset.y -= cmp::min(scroll, self.offset.y);
        self.align_cursor();
        self.print_screen();
    }

    fn handle_page_down(&mut self) {
        let scroll = rows() - 1; // Keep one line on screen
        let n = cmp::max(self.lines.len(), 1);
        let remaining = n - self.offset.y - 1;
        self.offset.y += cmp::min(scroll, remaining);
        if self.cursor.y + scroll > remaining {
            self.cursor.y = 0;
        }
        self.align_cursor();
        self.print_screen();
    }

    fn handle_arrow_up(&mut self) {
        if self.cursor.y > 0 {
            self.cursor.y -= 1
        } else if self.offset.y > 0 {
            self.offset.y -= 1;
        }
        self.align_cursor();
        self.print_screen();
    }

    fn handle_arrow_down(&mut self) {
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

    fn handle_ctrl_arrow_right(&mut self) {
        let tmp = self.search_query.clone();
        self.search_query = "\\w+".to_string();
        self.find_next();
        self.print_screen();
        self.search_query = tmp;
    }

    fn handle_ctrl_arrow_left(&mut self) {
        let tmp = self.search_query.clone();
        self.search_query = "\\w+".to_string();
        self.find_prev();
        self.print_screen();
        self.search_query = tmp;
    }

    fn cut_line(&mut self) {
        let i = self.offset.y + self.cursor.y;
        self.clipboard = Some(self.lines.remove(i));
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        if i == self.lines.len() {
            self.handle_arrow_up();
        } else {
            self.align_cursor();
            self.print_screen();
        }
    }

    fn copy_line(&mut self) {
        let i = self.offset.y + self.cursor.y;
        self.clipboard = Some(self.lines[i].clone());
    }

    fn paste_line(&mut self) {
        let i = self.offset.y + self.cursor.y;
        if let Some(line) = self.clipboard.clone() {
            self.lines.insert(i + 1, line);
            self.cursor.x = 0;
            self.offset.x = 0;
            self.handle_arrow_down(); // Move cursor to pasted line
        }
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
            Some('w') | Some('o') =>  {
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
            "o" | "open" if params.len() == 2 => { // Open
                self.open_buffer(params[1]);
                res = Some(Cmd::Open);
            }
            "q" | "quit" if params.len() == 1 => { // Quit
                res = Some(Cmd::Quit);
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
            "w" | "write" => { // Save file
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
        let re = Regex::new(&self.search_query);
        let dx = self.offset.x + self.cursor.x;
        let dy = self.offset.y + self.cursor.y;
        for (y, line) in self.lines.iter().enumerate() {
            let mut j = 0;
            if y < dy {
                continue;
            }
            if y == dy {
                j = cmp::min(dx, line.len());
                if let Some((i, end)) = re.find(&line[j..]) {
                    if i == 0 {
                        j += end; // Skip past current match
                    }
                }
            }
            if let Some((i, _)) = re.find(&line[j..]) {
                let x = j + i;
                self.cursor.x = x % cols();
                self.cursor.y = y % rows();
                self.offset.x = x - self.cursor.x;
                self.offset.y = y - self.cursor.y;
                break;
            }
        }
    }

    pub fn find_prev(&mut self) {
        let re = Regex::new(&self.search_query);
        let dx = self.offset.x + self.cursor.x;
        let dy = self.offset.y + self.cursor.y;
        for (y, line) in self.lines.iter().enumerate().rev() {
            let mut j = line.len();
            if y > dy {
                continue;
            }
            if y == dy {
                j = cmp::min(dx, line.len());
                if let Some((i, end)) = re.find(&line[..j]) {
                    if end == j {
                        j = i;
                    }
                }
            }
            if let Some((i, _)) = re.rfind(&line[..j]) {
                let x = i;
                self.cursor.x = x % cols();
                self.cursor.y = y % rows();
                self.offset.x = x - self.cursor.x;
                self.offset.y = y - self.cursor.y;
                break;
            }
        }
    }

    pub fn open(&mut self) {
        if let Some(path) = prompt(&mut self.buffer_prompt, "Open: ") {
            if !path.is_empty() {
                self.buffer_prompt.history.add(&path);
                self.open_buffer(&path);
            }
        }
    }

    pub fn open_buffer(&mut self, path: &str) {
        // Copy current buffer
        self.buffers[self.buf] = Buffer::from(&*self);

        // Open new buffer
        let buffer = Buffer::from(path);
        self.load_buffer(&buffer);
        self.buf += 1;
        self.buffers.insert(self.buf, buffer);
    }

    pub fn next_buffer(&mut self) {
        self.buffers[self.buf] = Buffer::from(&*self);
        self.buf = (self.buf + 1) % self.buffers.len();
        self.load_buffer(&self.buffers[self.buf].clone());
    }

    pub fn previous_buffer(&mut self) {
        self.buffers[self.buf] = Buffer::from(&*self);
        if self.buffers.len() > 1 {
            if self.buf == 0 {
                self.buf = self.buffers.len();
            }
            self.buf -= 1;
        }
        self.load_buffer(&self.buffers[self.buf].clone());
    }

    pub fn kill_buffer(&mut self) {
        if self.buffers.len() > 1 {
            self.previous_buffer();
            self.buffers.remove((self.buf + 1) % self.buffers.len());
        }
    }

    pub fn load_buffer(&mut self, buffer: &Buffer) {
        self.lines = buffer.lines.clone();
        self.pathname = buffer.pathname.clone();
        self.cursor = buffer.cursor.clone();
        self.offset = buffer.offset.clone();
        self.highlighted = buffer.highlighted.clone();
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
        "{}Usage:{} edit {}<options> (<path>[:row[:col]])+{1}",
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
    let mut paths = Vec::new();
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
                    error!("Invalid option {:?}", args[i]);
                    return Err(ExitCode::UsageError);
                } else {
                    paths.push(args[i])
                }
            }
        }
        i += 1;
    }
    if paths.is_empty() {
        help();
        return Err(ExitCode::UsageError);
    }

    let mut editor = Editor::new(paths[0]);
    let n = paths.len();
    for i in 1..n {
        editor.open_buffer(paths[i]);
    }

    if !cmd.is_empty() {
        for _ in 0..n {
            editor.next_buffer();
            editor.exec_command(cmd);
            for line in &editor.lines {
                println!("{}", line);
            }
        }
        return Ok(());
    }

    editor.run()
}
