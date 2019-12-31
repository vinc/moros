use crate::{print, user, kernel};
use heapless::{String, FnvIndexSet, Vec};
use heapless::consts::*;

pub struct Shell {
    cmd: String<U256>,
    history: FnvIndexSet<String<U256>, U256>,
    history_index: usize,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            cmd: String::new(),
            history: FnvIndexSet::new(),
            history_index: 0,
        }
    }

    pub fn run(&mut self) {
        self.print_prompt();
        loop {
            match kernel::console::get_char() {
                '\0' => {
                    continue;
                }
                '\n' => {
                    print!("\n");
                    if self.history.len() == self.history.capacity() {
                        let first = self.history.iter().next().unwrap().clone();
                        self.history.remove(&first);
                    }
                    if self.history.insert((self.cmd).clone()).is_ok() {
                        self.history_index = self.history.len();
                    }

                    if self.cmd.len() > 0 {
                        let line = self.cmd.clone();
                        let args: Vec<&str, U256> = line.split_whitespace().collect();
                        match args[0] {
                            "a" | "alias"                       => print!("TODO\n"),
                            "b"                                 => print!("?\n"),
                            "c" | "copy" | "cp"                 => print!("TODO\n"),
                            "d" | "del" | "delete" | "rm"       => print!("TODO\n"),
                            "e" | "edit"                        => print!("TODO\n"),
                            "f" | "find"                        => print!("TODO\n"),
                            "g" | "gd" | "go" | "go-dir" | "cd" => print!("TODO\n"),
                            "h" | "help"                        => print!("RTFM!\n"),
                            "i"                                 => print!("?\n"),
                            "j" | "jd" | "jump" | "jump-dir"    => print!("TODO\n"),
                            "k" | "kill"                        => print!("TODO\n"),
                            "l" | "list" | "ls"                 => print!("TODO\n"), // same as `rd`
                            "m" | "move" | "mv"                 => user::r#move::main(&args),
                            "n"                                 => print!("?\n"),
                            "o"                                 => print!("?\n"),
                            "p" | "print"                       => print!("TODO\n"),
                            "q" | "quit" | "exit"               => { return },
                            "r" | "read" | "cat"                => user::read::main(&args),
                            "s"                                 => print!("?\n"),
                            "t" | "tag"                         => print!("TODO\n"),
                            "u"                                 => print!("?\n"),
                            "v"                                 => print!("?\n"),
                            "w" | "write"                       => user::write::main(&args),
                            "x"                                 => print!("?\n"),
                            "y"                                 => print!("?\n"),
                            "z"                                 => print!("?\n"),
                            "rd" | "read-dir"                   => print!("TODO\n"),
                            "wd" | "write-dir" | "mkdir"        => print!("TODO\n"),
                            "sleep"                             => user::sleep::main(&args),
                            "clear"                             => kernel::vga::clear_screen(),
                            _ => print!("?\n"),
                        }
                        self.cmd.clear();
                    }
                    self.print_prompt();
                },
                '\x08' => { // Backspace
                    if self.cmd.len() > 0 {
                        self.cmd.pop();
                        print!("\x08");
                    }
                },
                '↑' => { // Arrow up
                    if self.history.len() > 0 {
                        if self.history_index > 0 {
                            self.history_index -= 1;
                        }
                        if let Some(cmd) = self.history.iter().nth(self.history_index) {
                            let n = self.cmd.len();
                            for _ in 0..n {
                                print!("\x08");
                            }
                            self.cmd = cmd.clone();
                            print!("{}", cmd);
                        }
                    }
                },
                '↓' => { // Arrow down
                    if self.history.len() > 0 {
                        if self.history_index < self.history.len() - 1 {
                            self.history_index += 1;
                        }
                        if let Some(cmd) = self.history.iter().nth(self.history_index) {
                            let n = self.cmd.len();
                            for _ in 0..n {
                                print!("\x08");
                            }
                            self.cmd = cmd.clone();
                            print!("{}", self.cmd);
                        }
                    }
                },
                c => {
                    if c.is_ascii_graphic() || c.is_ascii_whitespace() {
                        if self.cmd.push(c).is_ok() {
                            print!("{}", c);
                        }
                    }
                },
            }
        }
    }

    fn print_prompt(&self) {
        print!("\n> ");
    }
}
