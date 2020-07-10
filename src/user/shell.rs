use crate::{print, user, kernel};
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use alloc::string::String;

// TODO: Scan /bin
const AUTOCOMPLETE_COMMANDS: [&str; 29] = [
    "base64", "clear", "colors", "copy", "delete", "dhcp", "disk", "edit",
    "geotime", "goto", "halt", "help", "hex", "host", "http", "install", "ip",
    "list", "move", "net", "print", "quit", "read", "route", "shell", "sleep",
    "tcp", "user", "write",
];

// TODO: Scan /dev
const AUTOCOMPLETE_DEVICES: [&str; 5] = [
    "/dev/ata",
    "/dev/clk",
    "/dev/clk/uptime",
    "/dev/clk/realtime",
    "/dev/rtc",
];

#[repr(u8)]
#[derive(PartialEq)]
pub enum ExitCode {
    CommandSuccessful = 0,
    CommandUnknown    = 1,
    CommandError      = 2,
    ShellExit         = 255,
}

pub struct Shell {
    cmd: String,
    prompt: String,
    history: Vec<String>,
    history_index: usize,
    autocomplete: Vec<String>,
    autocomplete_index: usize,
    errored: bool,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            cmd: String::new(),
            prompt: String::from("> "),
            history: Vec::new(),
            history_index: 0,
            autocomplete: Vec::new(),
            autocomplete_index: 0,
            errored: false,
        }
    }

    pub fn run(&mut self) -> user::shell::ExitCode {
        self.load_history();
        print!("\n");
        self.print_prompt();
        loop {
            let (x, y) = kernel::vga::cursor_position();
            let c = kernel::console::get_char();
            match c {
                '\0' => {
                    continue;
                }
                '\x04' => { // Ctrl D
                    if self.cmd.is_empty() {
                        kernel::vga::clear_screen();
                        return ExitCode::CommandSuccessful;
                    }
                },
                '\x03' => { // Ctrl C
                    self.cmd.clear();
                    self.errored = false;
                    print!("\n\n");
                    self.print_prompt();
                },
                '\n' => { // Newline
                    self.update_history();
                    self.update_autocomplete();
                    print!("\n");
                    if self.cmd.len() > 0 {
                        // Add or move command to history at the end
                        let cmd = self.cmd.clone();
                        if let Some(pos) = self.history.iter().position(|s| *s == *cmd) {
                            self.history.remove(pos);
                        }
                        self.history.push(cmd);
                        self.history_index = self.history.len();

                        let line = self.cmd.clone();
                        match self.exec(&line) {
                            ExitCode::CommandSuccessful => {
                                self.errored = false;
                                self.save_history();
                            },
                            ExitCode::ShellExit => {
                                kernel::vga::clear_screen();
                                return ExitCode::CommandSuccessful
                            },
                            _ => {
                                self.errored = true;
                            },
                        }
                        self.cmd.clear();
                    } else {
                        self.errored = false;
                    }
                    print!("\n");
                    self.print_prompt();
                },
                '\t' => { // Tab
                    self.update_history();
                    self.print_autocomplete();
                },
                '↑' => { // Arrow up
                    self.update_autocomplete();
                    if self.history.len() > 0 {
                        if self.history_index > 0 {
                            self.history_index -= 1;
                        }
                        let cmd = &self.history[self.history_index];
                        let n = self.prompt.len();
                        kernel::console::clear_row_after(n + cmd.len());
                        kernel::vga::set_writer_position(n, y);
                        print!("{}", cmd);
                    }
                },
                '↓' => { // Arrow down
                    self.update_autocomplete();
                    if self.history_index < self.history.len() {
                        self.history_index += 1;
                        let cmd = if self.history_index < self.history.len() {
                            &self.history[self.history_index]
                        } else {
                            &self.cmd
                        };
                        let n = self.prompt.len();
                        kernel::console::clear_row_after(n + cmd.len());
                        kernel::vga::set_writer_position(n, y);
                        print!("{}", cmd);
                    }
                },
                '←' => { // Arrow left
                    self.update_history();
                    self.update_autocomplete();
                    if x > self.prompt.len() {
                        kernel::vga::set_cursor_position(x - 1, y);
                        kernel::vga::set_writer_position(x - 1, y);
                    }
                },
                '→' => { // Arrow right
                    self.update_history();
                    self.update_autocomplete();
                    if x < self.prompt.len() + self.cmd.len() {
                        kernel::vga::set_cursor_position(x + 1, y);
                        kernel::vga::set_writer_position(x + 1, y);
                    }
                },
                '\x08' => { // Backspace
                    self.update_history();
                    self.update_autocomplete();
                    if self.cmd.len() > 0 {
                        if kernel::console::has_cursor() {
                            if x > self.prompt.len() {
                                let cmd = self.cmd.clone();
                                let (before_cursor, mut after_cursor) = cmd.split_at(x - 1 - self.prompt.len());
                                if after_cursor.len() > 0 {
                                    after_cursor = &after_cursor[1..];
                                }
                                self.cmd.clear();
                                self.cmd.push_str(before_cursor);
                                self.cmd.push_str(after_cursor);
                                print!("{}{} ", c, after_cursor);
                                kernel::vga::set_cursor_position(x - 1, y);
                                kernel::vga::set_writer_position(x - 1, y);
                            }
                        } else {
                            self.cmd.pop();
                            print!("{}", c);
                        }
                    }
                },
                c => {
                    self.update_history();
                    self.update_autocomplete();
                    if c.is_ascii() && kernel::vga::is_printable(c as u8) {
                        if kernel::console::has_cursor() {
                            let cmd = self.cmd.clone();
                            let (before_cursor, after_cursor) = cmd.split_at(x - self.prompt.len());
                            self.cmd.clear();
                            self.cmd.push_str(before_cursor);
                            self.cmd.push(c);
                            self.cmd.push_str(after_cursor);
                            print!("{}{}", c, after_cursor);
                            kernel::vga::set_cursor_position(x + 1, y);
                            kernel::vga::set_writer_position(x + 1, y);
                        } else {
                            self.cmd.push(c);
                            print!("{}", c);
                        }
                    }
                },
            }
        }
    }

    // Called when a key other than up or down is pressed while in history
    // mode. The history index point to a command that will be selected and
    // the index will be reset to the length of the history vector to signify
    // that the editor is no longer in history mode.
    pub fn update_history(&mut self) {
        if self.history_index != self.history.len() {
            self.cmd = self.history[self.history_index].clone();
            self.history_index = self.history.len();
        }
    }

    pub fn load_history(&mut self) {
        if let Some(home) = kernel::process::env("HOME") {
            let pathname = format!("{}/.shell_history", home);

            if let Some(file) = kernel::fs::File::open(&pathname) {
                let contents = file.read_to_string();
                for line in contents.split('\n') {
                    let cmd = line.trim();
                    if cmd.len() > 0 {
                        self.history.push(cmd.into());
                    }
                }
            }
            self.history_index = self.history.len();
        }
    }

    pub fn save_history(&mut self) {
        if let Some(home) = kernel::process::env("HOME") {
            let pathname = format!("{}/.shell_history", home);

            let mut contents = String::new();
            for cmd in &self.history {
                contents.push_str(&format!("{}\n", cmd));
            }

            let mut file = match kernel::fs::File::open(&pathname) {
                Some(file) => file,
                None => kernel::fs::File::create(&pathname).unwrap(),
            };

            file.write(&contents.as_bytes()).unwrap();
        }
    }

    pub fn print_autocomplete(&mut self) {
        let mut args = self.parse(&self.cmd);
        let i = args.len() - 1;
        if self.autocomplete_index == 0 {
            if args.len() == 1 { // Autocomplete cmd
                self.autocomplete = vec![args[i].into()];
                for &cmd in &AUTOCOMPLETE_COMMANDS {
                    if cmd.starts_with(args[i]) {
                        self.autocomplete.push(cmd.into());
                    }
                }
            } else { // Autocomplete path
                let pathname = kernel::fs::realpath(args[i]);
                let dirname = kernel::fs::dirname(&pathname);
                let filename = kernel::fs::filename(&pathname);
                self.autocomplete = vec![args[i].into()];
                let sep = if dirname.ends_with("/") { "" } else { "/" };
                if pathname.starts_with("/dev") {
                    for dev in &AUTOCOMPLETE_DEVICES {
                        let d = kernel::fs::dirname(dev);
                        let f = kernel::fs::filename(dev);
                        if d == dirname && f.starts_with(filename) {
                            self.autocomplete.push(format!("{}{}{}", d, sep, f));
                        }
                    }
                } else if let Some(dir) = kernel::fs::Dir::open(dirname) {
                    for entry in dir.read() {
                        if entry.name().starts_with(filename) {
                            self.autocomplete.push(format!("{}{}{}", dirname, sep, entry.name()));
                        }
                    }
                }
            }
        }

        self.autocomplete_index = (self.autocomplete_index + 1) % self.autocomplete.len();
        args[i] = &self.autocomplete[self.autocomplete_index];

        let cmd = args.join(" ");
        let n = self.prompt.len();
        let (_, y) = kernel::console::cursor_position();
        kernel::console::clear_row_after(n + cmd.len());
        kernel::console::set_writer_position(n, y);
        print!("{}", cmd);
    }

    // Called when a key other than tab is pressed while in autocomplete mode.
    // The autocomplete index point to an argument that will be added to the
    // command and the index will be reset to signify that the editor is no
    // longer in autocomplete mode.
    pub fn update_autocomplete(&mut self) {
        if self.autocomplete_index != 0 {
            let mut args = self.parse(&self.cmd);
            let i = args.len() - 1;
            args[i] = &self.autocomplete[self.autocomplete_index];
            self.cmd = args.join(" ");
            self.autocomplete_index = 0;
            self.autocomplete = vec!["".into()];
        }
    }

    pub fn parse<'a>(&self, cmd: &'a str) -> Vec<&'a str> {
        //let args: Vec<&str> = cmd.split_whitespace().collect();
        let mut args: Vec<&str> = Vec::new();
        let mut i = 0;
        let mut n = cmd.len();
        let mut is_quote = false;

        for (j, c) in cmd.char_indices() {
            if c == '#' && !is_quote {
                n = j; // Discard comments
                break;
            } else if c == ' ' && !is_quote {
                if i != j {
                    args.push(&cmd[i..j]);
                }
                i = j + 1;
            } else if c == '"' {
                is_quote = !is_quote;
                if !is_quote {
                    args.push(&cmd[i..j]);
                }
                i = j + 1;
            }
        }

        if i < n {
            if is_quote {
                n -= 1;
            }
            args.push(&cmd[i..n]);
        }

        if n == 0 || cmd.chars().last().unwrap() == ' ' {
            args.push("");
        }

        args
    }

    pub fn exec(&self, cmd: &str) -> ExitCode {
        let args = self.parse(cmd);

        match args[0] {
            ""                     => ExitCode::CommandSuccessful,
            "a" | "alias"          => ExitCode::CommandUnknown,
            "b"                    => ExitCode::CommandUnknown,
            "c" | "copy"           => user::copy::main(&args),
            "d" | "del" | "delete" => user::delete::main(&args),
            "e" | "edit"           => user::editor::main(&args),
            "f" | "find"           => ExitCode::CommandUnknown,
            "g" | "go" | "goto"    => self.change_dir(&args),
            "h" | "help"           => user::help::main(&args),
            "i"                    => ExitCode::CommandUnknown,
            "j" | "jump"           => ExitCode::CommandUnknown,
            "k" | "kill"           => ExitCode::CommandUnknown,
            "l" | "list"           => user::list::main(&args),
            "m" | "move"           => user::r#move::main(&args),
            "n"                    => ExitCode::CommandUnknown,
            "o"                    => ExitCode::CommandUnknown,
            "p" | "print"          => user::print::main(&args),
            "q" | "quit" | "exit"  => ExitCode::ShellExit,
            "r" | "read"           => user::read::main(&args),
            "s"                    => ExitCode::CommandUnknown,
            "t"                    => ExitCode::CommandUnknown,
            "u"                    => ExitCode::CommandUnknown,
            "v"                    => ExitCode::CommandUnknown,
            "w" | "write"          => user::write::main(&args),
            "x"                    => ExitCode::CommandUnknown,
            "y"                    => ExitCode::CommandUnknown,
            "z"                    => ExitCode::CommandUnknown,
            "shell"                => user::shell::main(&args),
            "sleep"                => user::sleep::main(&args),
            "clear"                => user::clear::main(&args),
            "base64"               => user::base64::main(&args),
            "halt"                 => user::halt::main(&args),
            "hex"                  => user::hex::main(&args), // TODO: Rename to `dump`
            "net"                  => user::net::main(&args),
            "route"                => user::route::main(&args),
            "dhcp"                 => user::dhcp::main(&args),
            "http"                 => user::http::main(&args),
            "tcp"                  => user::tcp::main(&args),
            "host"                 => user::host::main(&args),
            "install"              => user::install::main(&args),
            "ip"                   => user::ip::main(&args),
            "geotime"              => user::geotime::main(&args),
            "colors"               => user::colors::main(&args),
            "disk"                 => user::disk::main(&args),
            "user"                 => user::user::main(&args),
            _                      => ExitCode::CommandUnknown,
        }
    }

    fn print_prompt(&self) {
        let color = if self.errored { "Red" } else { "Magenta" };
        let csi_color = kernel::console::color(color);
        let csi_reset = kernel::console::color("Reset");
        print!("{}{}{}", csi_color, self.prompt, csi_reset);
    }

    fn change_dir(&self, args: &[&str]) -> ExitCode {
        match args.len() {
            1 => {
                print!("{}\n", kernel::process::dir());
                ExitCode::CommandSuccessful
            },
            2 => {
                let pathname = kernel::fs::realpath(args[1]);
                if kernel::fs::Dir::open(&pathname).is_some() {
                    kernel::process::set_dir(&pathname);
                    ExitCode::CommandSuccessful
                } else {
                    print!("File not found '{}'\n", pathname);
                    ExitCode::CommandError
                }
            },
            _ => {
                ExitCode::CommandError
            }
        }
    }
}

pub fn main(args: &[&str]) -> ExitCode {
    let mut shell = Shell::new();
    match args.len() {
        1 => {
            return shell.run();
        },
        2 => {
            let pathname = args[1];
            if let Some(file) = kernel::fs::File::open(pathname) {
                for line in file.read_to_string().split("\n") {
                    if line.len() > 0 {
                        shell.exec(line);
                    }
                }
                ExitCode::CommandSuccessful
            } else {
                print!("File not found '{}'\n", pathname);
                ExitCode::CommandError
            }
        },
        _ => {
            ExitCode::CommandError
        },
    }
}
