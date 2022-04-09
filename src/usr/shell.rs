use crate::{api, sys, usr};
use crate::api::fs;
use crate::api::regex::Regex;
use crate::api::prompt::Prompt;
use crate::api::console::Style;
use alloc::format;
use alloc::vec::Vec;
use alloc::string::String;

// TODO: Scan /bin
const AUTOCOMPLETE_COMMANDS: [&str; 37] = [
    "2048", "base64", "calc", "clear", "colors", "copy", "date", "delete", "dhcp", "disk", "edit",
    "env", "exit", "geotime", "goto", "halt", "help", "hex", "host", "http", "httpd", "install",
    "keyboard", "lisp", "list", "memory", "move", "net", "pci", "print", "read", "shell", "sleep",
    "tcp", "user", "vga", "write"
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
        let pathname = fs::realpath(args[i]);
        let dirname = fs::dirname(&pathname);
        let filename = fs::filename(&pathname);
        let sep = if dirname.ends_with('/') { "" } else { "/" };
        if let Ok(files) = fs::read_dir(dirname) {
            for file in files {
                let name = file.name();
                if name.starts_with(filename) {
                    let end = if file.is_dir() { "/" } else { "" };
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

fn proc(args: &[&str]) -> ExitCode {
    match args.len() {
        1 => {
            ExitCode::CommandSuccessful
        },
        2 => {
            match args[1] {
                "id" => {
                    println!("{}", sys::process::id());
                    ExitCode::CommandSuccessful
                }
                "files" => {
                    for (i, handle) in sys::process::file_handles().iter().enumerate() {
                        if let Some(resource) = handle {
                            println!("{}: {:?}", i, resource);
                        }
                    }
                    ExitCode::CommandSuccessful
                }
                _ => {
                    ExitCode::CommandError
                }
            }
        },
        _ => {
            ExitCode::CommandError
        }
    }
}

fn change_dir(args: &[&str]) -> ExitCode {
    match args.len() {
        1 => {
            println!("{}", sys::process::dir());
            ExitCode::CommandSuccessful
        },
        2 => {
            let mut pathname = fs::realpath(args[1]);
            if pathname.len() > 1 {
                pathname = pathname.trim_end_matches('/').into();
            }
            if api::fs::is_dir(&pathname) {
                sys::process::set_dir(&pathname);
                ExitCode::CommandSuccessful
            } else {
                error!("File not found '{}'", pathname);
                ExitCode::CommandError
            }
        },
        _ => {
            ExitCode::CommandError
        }
    }
}

pub fn exec(cmd: &str) -> ExitCode {
    let mut args = split_args(cmd);

    // Redirections like `print hello => /tmp/hello`
    // Pipes like `print hello -> write /tmp/hello` or `p hello > w /tmp/hello`
    let mut is_redirected = false;
    let mut n = args.len();
    let mut i = 0;
    loop {
        if i == n {
            break;
        }

        let mut is_fat_arrow = false;
        let mut is_thin_arrow = false;
        let mut left_handle;

        if Regex::new("<=+").is_match(args[i]) { // Redirect input stream
            is_fat_arrow = true;
            left_handle = 0;
        } else if Regex::new("\\d*=+>").is_match(args[i]) { // Redirect output stream(s)
            is_fat_arrow = true;
            left_handle = 1;
        } else if Regex::new("\\d*-*>\\d*").is_match(args[i]) { // Pipe output stream(s)
            is_thin_arrow = true;
            left_handle = 1;
            // TODO: right_handle?
        } else {
            i += 1;
            continue;
        }

        let s = args[i].chars().take_while(|c| c.is_numeric()).collect::<String>();
        if let Ok(h) = s.parse() {
            left_handle = h;
        }

        if is_fat_arrow { // Redirections
            is_redirected = true;
            if i == n - 1 {
                println!("Could not parse path for redirection");
                return ExitCode::CommandError;
            }
            let path = args[i + 1];
            if api::fs::reopen(path, left_handle).is_err() {
                println!("Could not open path for redirection");
                return ExitCode::CommandError;
            }
            args.remove(i); // Remove redirection from args
            args.remove(i); // Remove path from args
            n -= 2;
        } else if is_thin_arrow { // TODO: Implement pipes
            println!("Could not parse arrow");
            return ExitCode::CommandError;
        }
    }

    let res = match args[0] {
        ""                     => ExitCode::CommandSuccessful,
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
        "sh" | "shell"         => usr::shell::main(&args),
        "sleep"                => usr::sleep::main(&args),
        "clear"                => usr::clear::main(&args),
        "calc"                 => usr::calc::main(&args),
        "base64"               => usr::base64::main(&args),
        "date"                 => usr::date::main(&args),
        "env"                  => usr::env::main(&args),
        "halt"                 => usr::halt::main(&args),
        "hex"                  => usr::hex::main(&args),
        "net"                  => usr::net::main(&args),
        "dhcp"                 => usr::dhcp::main(&args),
        "http"                 => usr::http::main(&args),
        "httpd"                => usr::httpd::main(&args),
        "tcp"                  => usr::tcp::main(&args),
        "host"                 => usr::host::main(&args),
        "install"              => usr::install::main(&args),
        "geotime"              => usr::geotime::main(&args),
        "colors"               => usr::colors::main(&args),
        "dsk" | "disk"         => usr::disk::main(&args),
        "user"                 => usr::user::main(&args),
        "mem" | "memory"       => usr::memory::main(&args),
        "kb" | "keyboard"      => usr::keyboard::main(&args),
        "lisp"                 => usr::lisp::main(&args),
        "chess"                => usr::chess::main(&args),
        "beep"                 => usr::beep::main(&args),
        "elf"                  => usr::elf::main(&args),
        "pci"                  => usr::pci::main(&args),
        "2048"                 => usr::pow::main(&args),
        "proc"                 => proc(&args),
        cmd                    => {
            if api::process::spawn(cmd).is_ok() {
                ExitCode::CommandSuccessful
            } else {
                ExitCode::CommandUnknown
            }
        }
    };

    // TODO: Remove this when redirections are done in spawned process
    if is_redirected {
        for i in 0..3 {
            api::fs::reopen("/dev/console", i).ok();
        }
    }

    res
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
            },
            ExitCode::ShellExit => {
                break;
            },
            _ => {
                success = false;
            },
        }
        prompt.history.add(&cmd);
        prompt.history.save(history_file);
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
            if let Ok(contents) = api::fs::read_to_string(pathname) {
                for line in contents.split('\n') {
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

#[test_case]
fn test_shell() {
    use alloc::string::ToString;

    sys::fs::mount_mem();
    sys::fs::format_mem();
    usr::install::copy_files(false);


    // Redirect standard output
    exec("print test1 => /test");
    assert_eq!(api::fs::read_to_string("/test"), Ok("test1\n".to_string()));

    // Overwrite content of existing file
    exec("print test2 => /test");
    assert_eq!(api::fs::read_to_string("/test"), Ok("test2\n".to_string()));

    // Redirect standard output explicitely
    exec("print test3 1=> /test");
    assert_eq!(api::fs::read_to_string("/test"), Ok("test3\n".to_string()));

    // Redirect standard error explicitely
    exec("hex /nope 2=> /test");
    assert!(api::fs::read_to_string("/test").unwrap().contains("File not found '/nope'"));

    sys::fs::dismount();
}
