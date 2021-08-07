use crate::api::prompt::Prompt;
use crate::{sys, usr};
use crate::api::console::Style;
use alloc::format;
use alloc::vec::Vec;
use alloc::string::String;

// TODO: Scan /bin
const AUTOCOMPLETE_COMMANDS: [&str; 36] = [
    "base64", "clear", "colors", "copy", "date", "delete", "dhcp", "disk", "edit", "env", "exit",
    "geotime", "goto", "halt", "help", "hex", "host", "http", "httpd", "install", "ip", "keyboard",
    "lisp", "list", "memory", "move", "net", "print", "read", "route", "shell", "sleep", "tcp",
    "user", "vga", "write"
];

#[repr(u8)]
#[derive(PartialEq)]
pub enum ExitCode {
    CommandSuccessful = 0,
    CommandUnknown    = 1,
    CommandError      = 2,
    ShellExit         = 255,
}

fn shell_completer(line: &str) -> Vec<String> {
    let mut entries = Vec::new();

    let args = split_args(line);
    let i = args.len() - 1;
    if args.len() == 1 { // Autocomplete command
        for &cmd in &AUTOCOMPLETE_COMMANDS {
            if let Some(entry) = cmd.strip_prefix(args[i]) {
                entries.push(entry.into());
            }
        }
    } else { // Autocomplete path
        let pathname = sys::fs::realpath(args[i]);
        let dirname = sys::fs::dirname(&pathname);
        let filename = sys::fs::filename(&pathname);
        let sep = if dirname.ends_with('/') { "" } else { "/" };
        if let Some(dir) = sys::fs::Dir::open(dirname) {
            for entry in dir.read() {
                let name = entry.name();
                if name.starts_with(filename) {
                    let end = if entry.is_dir() { "/" } else { "" };
                    let path = format!("{}{}{}{}", dirname, sep, name, end);
                    entries.push(path[pathname.len()..].into());
                }
            }
        }
    }
    entries
}

pub fn prompt_string(success: bool) -> String {
    let csi_color = Style::color("Magenta");
    let csi_error = Style::color("Red");
    let csi_reset = Style::reset();
    format!("{}>{} ", if success { csi_color } else { csi_error }, csi_reset)
}

pub fn split_args(cmd: &str) -> Vec<&str> {
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

    if n == 0 || cmd.ends_with(' ') {
        args.push("");
    }

    args
}

fn change_dir(args: &[&str]) -> ExitCode {
    match args.len() {
        1 => {
            println!("{}", sys::process::dir());
            ExitCode::CommandSuccessful
        },
        2 => {
            let mut pathname = sys::fs::realpath(args[1]);
            if pathname.len() > 1 {
                pathname = pathname.trim_end_matches('/').into();
            }
            if sys::fs::Dir::open(&pathname).is_some() {
                sys::process::set_dir(&pathname);
                ExitCode::CommandSuccessful
            } else {
                println!("File not found '{}'", pathname);
                ExitCode::CommandError
            }
        },
        _ => {
            ExitCode::CommandError
        }
    }
}

pub fn exec(cmd: &str) -> ExitCode {
    let args = split_args(cmd);

    match args[0] {
        ""                     => ExitCode::CommandError,
        "a" | "alias"          => ExitCode::CommandUnknown,
        "b"                    => ExitCode::CommandUnknown,
        "c" | "copy"           => usr::copy::main(&args),
        "d" | "del" | "delete" => usr::delete::main(&args),
        "e" | "edit"           => usr::editor::main(&args),
        "f" | "find"           => usr::find::main(&args),
        "g" | "go" | "goto"    => change_dir(&args),
        "h" | "help"           => usr::help::main(&args),
        "i"                    => ExitCode::CommandUnknown,
        "j" | "jump"           => ExitCode::CommandUnknown,
        "k" | "kill"           => ExitCode::CommandUnknown,
        "l" | "list"           => usr::list::main(&args),
        "m" | "move"           => usr::r#move::main(&args),
        "n"                    => ExitCode::CommandUnknown,
        "o"                    => ExitCode::CommandUnknown,
        "p" | "print"          => usr::print::main(&args),
        "q" | "quit" | "exit"  => ExitCode::ShellExit,
        "r" | "read"           => usr::read::main(&args),
        "s"                    => ExitCode::CommandUnknown,
        "t"                    => ExitCode::CommandUnknown,
        "u"                    => ExitCode::CommandUnknown,
        "v"                    => ExitCode::CommandUnknown,
        "w" | "write"          => usr::write::main(&args),
        "x"                    => ExitCode::CommandUnknown,
        "y"                    => ExitCode::CommandUnknown,
        "z"                    => ExitCode::CommandUnknown,
        "vga"                  => usr::vga::main(&args),
        "shell"                => usr::shell::main(&args),
        "sleep"                => usr::sleep::main(&args),
        "clear"                => usr::clear::main(&args),
        "base64"               => usr::base64::main(&args),
        "date"                 => usr::date::main(&args),
        "env"                  => usr::env::main(&args),
        "halt"                 => usr::halt::main(&args),
        "hex"                  => usr::hex::main(&args),
        "net"                  => usr::net::main(&args),
        "route"                => usr::route::main(&args),
        "dhcp"                 => usr::dhcp::main(&args),
        "http"                 => usr::http::main(&args),
        "httpd"                => usr::httpd::main(&args),
        "tcp"                  => usr::tcp::main(&args),
        "host"                 => usr::host::main(&args),
        "install"              => usr::install::main(&args),
        "ip"                   => usr::ip::main(&args),
        "geotime"              => usr::geotime::main(&args),
        "colors"               => usr::colors::main(&args),
        "dsk" | "disk"         => usr::disk::main(&args),
        "user"                 => usr::user::main(&args),
        "mem" | "memory"       => usr::mem::main(&args),
        "kb" | "keyboard"      => usr::keyboard::main(&args),
        "lisp"                 => usr::lisp::main(&args),
        "chess"                => usr::chess::main(&args),
        _                      => ExitCode::CommandUnknown,
    }
}

pub fn run() -> usr::shell::ExitCode {
    println!();

    let mut prompt = Prompt::new();
    let history_file = "~/.shell-history";
    prompt.history.load(history_file);
    prompt.completion.set(&shell_completer);

    let mut success = true;
    while let Some(cmd) = prompt.input(&prompt_string(success)) {
        match exec(&cmd) {
            ExitCode::CommandSuccessful => {
                success = true;
                prompt.history.add(&cmd);
                prompt.history.save(history_file);
            },
            ExitCode::ShellExit => {
                break;
            },
            _ => {
                success = false;
            },
        }
        sys::console::drain();
        println!();
    }
    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move cursor to top
    ExitCode::CommandSuccessful
}

pub fn main(args: &[&str]) -> ExitCode {
    match args.len() {
        1 => {
            run()
        },
        2 => {
            let pathname = args[1];
            if let Some(mut file) = sys::fs::File::open(pathname) {
                for line in file.read_to_string().split('\n') {
                    if !line.is_empty() {
                        exec(line);
                    }
                }
                ExitCode::CommandSuccessful
            } else {
                println!("File not found '{}'", pathname);
                ExitCode::CommandError
            }
        },
        _ => {
            ExitCode::CommandError
        },
    }
}
