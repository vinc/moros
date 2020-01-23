use crate::{print, user, kernel};
use alloc::vec::Vec;
use alloc::string::String;

#[repr(u8)]
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
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            cmd: String::new(),
            prompt: String::from("> "),
            history: Vec::new(),
            history_index: 0,
        }
    }

    pub fn run(&mut self) -> user::shell::ExitCode {
        self.print_prompt();
        loop {
            let (x, y) = kernel::vga::cursor_position();
            let c = kernel::console::get_char();
            match c {
                '\0' => {
                    continue;
                }
                '\x03' => { // Ctrl C
                    if self.cmd.len() > 0 {
                        self.cmd.clear();
                        print!("\n");
                        self.print_prompt();
                    } else {
                        return ExitCode::CommandSuccessful;
                    }
                },
                '\n' => { // Newline
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
                            ExitCode::CommandSuccessful => {},
                            ExitCode::ShellExit => { return ExitCode::CommandSuccessful },
                            _ => { print!("?\n") },
                        }
                        self.cmd.clear();
                    }
                    self.print_prompt();
                },
                '↑' => { // Arrow up
                    if self.history.len() > 0 {
                        if self.history_index > 0 {
                            self.history_index -= 1;
                        }
                        if let Some(cmd) = self.history.iter().nth(self.history_index) {
                            self.cmd = cmd.clone();
                            kernel::vga::clear_row();
                            print!("{}{}", self.prompt, self.cmd);
                        }
                    }
                },
                '↓' => { // Arrow down
                    if self.history.len() > 0 {
                        if self.history_index < self.history.len() - 1 {
                            self.history_index += 1;
                        }
                        if let Some(cmd) = self.history.iter().nth(self.history_index) {
                            self.cmd = cmd.clone();
                            kernel::vga::clear_row();
                            print!("{}{}", self.prompt, self.cmd);
                        }
                    }
                },
                '←' => { // Arrow left
                    if x > self.prompt.len() {
                        kernel::vga::set_cursor_position(x - 1, y);
                        kernel::vga::set_writer_position(x - 1, y);
                    }
                },
                '→' => { // Arrow right
                    if x < self.prompt.len() + self.cmd.len() {
                        kernel::vga::set_cursor_position(x + 1, y);
                        kernel::vga::set_writer_position(x + 1, y);
                    }
                },
                '\x08' => { // Backspace
                    let cmd = self.cmd.clone();
                    if cmd.len() > 0 && x > 0 {
                        let (before_cursor, mut after_cursor) = cmd.split_at(x - 1 - self.prompt.len());
                        if after_cursor.len() > 0 {
                            after_cursor = &after_cursor[1..];
                        }
                        self.cmd.clear();
                        self.cmd.push_str(before_cursor);
                        self.cmd.push_str(after_cursor);
                        kernel::vga::clear_row();
                        print!("{}{}", self.prompt, self.cmd);
                        kernel::vga::set_cursor_position(x - 1, y);
                        kernel::vga::set_writer_position(x - 1, y);
                    }
                },
                c => {
                    if c.is_ascii() && kernel::vga::is_printable(c as u8) {
                        let cmd = self.cmd.clone();
                        let (before_cursor, after_cursor) = cmd.split_at(x - self.prompt.len());
                        self.cmd.clear();
                        self.cmd.push_str(before_cursor);
                        self.cmd.push(c);
                        self.cmd.push_str(after_cursor);
                        kernel::vga::clear_row();
                        print!("{}{}", self.prompt, self.cmd);
                        kernel::vga::set_cursor_position(x + 1, y);
                        kernel::vga::set_writer_position(x + 1, y);
                    }
                },
            }
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

        args
    }

    pub fn exec(&self, cmd: &str) -> ExitCode {
        let args = self.parse(cmd);

        if args.len() == 0 {
            return ExitCode::CommandSuccessful;
        }

        match args[0] {
            "a" | "alias"          => ExitCode::CommandUnknown,
            "b"                    => ExitCode::CommandUnknown,
            "c" | "copy"           => user::copy::main(&args),
            "d" | "del" | "delete" => user::delete::main(&args),
            "e" | "edit"           => user::editor::main(&args),
            "f" | "find"           => ExitCode::CommandUnknown,
            "g" | "go"             => ExitCode::CommandUnknown,
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
            "login"                => user::login::main(&args),
            "base64"               => user::base64::main(&args),
            "halt"                 => user::halt::main(&args),
            "hex"                  => user::hex::main(&args),
            _                      => ExitCode::CommandUnknown,
        }
    }

    fn print_prompt(&self) {
        print!("\n{}", self.prompt);
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
