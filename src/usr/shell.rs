use crate::{api, sys, usr};
use crate::api::console::Style;
use crate::api::fs;
use crate::api::prompt::Prompt;
use crate::api::regex::Regex;
use crate::api::syscall;
use crate::sys::fs::FileType;

use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::vec::Vec;
use alloc::string::{String, ToString};

// TODO: Scan /bin
const AUTOCOMPLETE_COMMANDS: [&str; 35] = [
    "2048", "base64", "calc", "colors", "copy", "date", "delete", "dhcp", "disk", "edit",
    "env", "geotime", "goto", "help", "hex", "host", "http", "httpd", "install",
    "keyboard", "lisp", "list", "memory", "move", "net", "pci", "quit", "read",
    "shell", "socket", "tcp", "time", "user", "vga", "write"
];

struct Config {
    env: BTreeMap<String, String>,
    aliases: BTreeMap<String, String>,
}

impl Config {
    fn new() -> Config {
        let aliases = BTreeMap::new();
        let mut env = BTreeMap::new();
        for (key, val) in sys::process::envs() {
            env.insert(key, val); // Copy the process environment to the shell environment
        }
        env.insert("DIR".to_string(), sys::process::dir());
        Config { env, aliases }
    }
}

fn autocomplete_commands() -> Vec<String> {
    let mut res = Vec::new();
    for cmd in AUTOCOMPLETE_COMMANDS {
        res.push(cmd.to_string());
    }
    if let Ok(files) = fs::read_dir("/bin") {
        for file in files {
            res.push(file.name());
        }
    }
    res
}

fn shell_completer(line: &str) -> Vec<String> {
    let mut entries = Vec::new();

    let args = split_args(line);
    let i = args.len() - 1;
    if args.len() == 1 && !args[0].starts_with('/') { // Autocomplete command
        for cmd in autocomplete_commands() {
            if let Some(entry) = cmd.strip_prefix(&args[i]) {
                entries.push(entry.into());
            }
        }
    } else { // Autocomplete path
        let pathname = fs::realpath(&args[i]);
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
    entries.sort();
    entries
}

pub fn prompt_string(success: bool) -> String {
    let csi_color = Style::color("Magenta");
    let csi_error = Style::color("Red");
    let csi_reset = Style::reset();
    format!("{}>{} ", if success { csi_color } else { csi_error }, csi_reset)
}

fn is_globbing(arg: &str) -> bool {
    let arg: Vec<char> = arg.chars().collect();
    let n = arg.len();
    if n == 0 {
        return false;
    }
    if arg[0] == '"' && arg[n - 1] == '"' {
        return false;
    }
    if arg[0] == '\'' && arg[n - 1] == '\'' {
        return false;
    }
    for i in 0..n {
        if arg[i] == '*' || arg[i] == '?' {
            return true;
        }
    }
    false
}

fn glob_to_regex(pattern: &str) -> String {
    format!("^{}$", pattern
        .replace('\\', "\\\\") // `\` string literal
        .replace('.', "\\.") // `.` string literal
        .replace('*', ".*")  // `*` match zero or more chars except `/`
        .replace('?', ".")   // `?` match any char except `/`
    )
}

fn glob(arg: &str) -> Vec<String> {
    let mut matches = Vec::new();
    if is_globbing(arg) {
        let (dir, pattern) = if arg.contains('/') {
            (fs::dirname(arg).to_string(), fs::filename(arg).to_string())
        } else {
            (sys::process::dir(), arg.to_string())
        };

        let re = Regex::new(&glob_to_regex(&pattern));

        if let Ok(files) = fs::read_dir(&dir) {
            for file in files {
                let name = file.name();
                if re.is_match(&name) {
                    matches.push(format!("{}/{}", dir, name));
                }
            }
        }
    } else {
        matches.push(arg.to_string());
    }
    matches
}

pub fn split_args(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut i = 0;
    let mut n = cmd.len();
    let mut is_quote = false;

    for (j, c) in cmd.char_indices() {
        if c == '#' && !is_quote {
            n = j; // Discard comments
            break;
        } else if c == ' ' && !is_quote {
            if i != j {
                if args.is_empty() {
                    args.push(cmd[i..j].to_string())
                } else {
                    args.extend(glob(&cmd[i..j]))
                }
            }
            i = j + 1;
        } else if c == '"' {
            is_quote = !is_quote;
            if !is_quote {
                args.push(cmd[i..j].to_string());
            }
            i = j + 1;
        }
    }

    if i < n {
        if is_quote {
            n -= 1;
            args.push(cmd[i..n].to_string());
        } else if args.is_empty() {
            args.push(cmd[i..n].to_string());
        } else {
            args.extend(glob(&cmd[i..n]))
        }
    }

    if n == 0 || cmd.ends_with(' ') {
        args.push("".to_string());
    }

    args
}

fn cmd_proc(args: &[&str]) -> Result<usize, usize> {
    match args.len() {
        1 => {
            Ok(0)
        },
        2 => {
            match args[1] {
                "id" => {
                    println!("{}", sys::process::id());
                    Ok(0)
                }
                "files" => {
                    for (i, handle) in sys::process::file_handles().iter().enumerate() {
                        if let Some(resource) = handle {
                            println!("{}: {:?}", i, resource);
                        }
                    }
                    Ok(0)
                }
                _ => {
                    Err(1)
                }
            }
        },
        _ => {
            Err(1)
        }
    }
}

fn cmd_change_dir(args: &[&str], config: &mut Config) -> Result<usize, usize> {
    match args.len() {
        1 => {
            println!("{}", sys::process::dir());
            Ok(0)
        },
        2 => {
            let mut pathname = fs::realpath(args[1]);
            if pathname.len() > 1 {
                pathname = pathname.trim_end_matches('/').into();
            }
            if api::fs::is_dir(&pathname) {
                sys::process::set_dir(&pathname);
                config.env.insert("DIR".to_string(), sys::process::dir());
                Ok(0)
            } else {
                error!("File not found '{}'", pathname);
                Err(1)
            }
        },
        _ => {
            Err(1)
        }
    }
}

fn cmd_alias(args: &[&str], config: &mut Config) -> Result<usize, usize> {
    if args.len() != 3 {
        let csi_option = Style::color("LightCyan");
        let csi_title = Style::color("Yellow");
        let csi_reset = Style::reset();
        println!("{}Usage:{} alias {}<key> <val>{1}", csi_title, csi_reset, csi_option);
        return Err(1);
    }
    config.aliases.insert(args[1].to_string(), args[2].to_string());
    Ok(0)
}

fn cmd_unalias(args: &[&str], config: &mut Config) -> Result<usize, usize> {
    if args.len() != 2 {
        let csi_option = Style::color("LightCyan");
        let csi_title = Style::color("Yellow");
        let csi_reset = Style::reset();
        println!("{}Usage:{} unalias {}<key>{1}", csi_title, csi_reset, csi_option);
        return Err(1);
    }

    if config.aliases.remove(&args[1].to_string()).is_none() {
        error!("Error: could not unalias '{}'", args[1]);
        return Err(1);
    }

    Ok(0)
}

fn cmd_set(args: &[&str], config: &mut Config) -> Result<usize, usize> {
    if args.len() != 3 {
        let csi_option = Style::color("LightCyan");
        let csi_title = Style::color("Yellow");
        let csi_reset = Style::reset();
        println!("{}Usage:{} set {}<key> <val>{1}", csi_title, csi_reset, csi_option);
        return Err(1);
    }

    config.env.insert(args[1].to_string(), args[2].to_string());
    Ok(0)
}

fn cmd_unset(args: &[&str], config: &mut Config) -> Result<usize, usize> {
    if args.len() != 2 {
        let csi_option = Style::color("LightCyan");
        let csi_title = Style::color("Yellow");
        let csi_reset = Style::reset();
        println!("{}Usage:{} unset {}<key>{1}", csi_title, csi_reset, csi_option);
        return Err(1);
    }

    if config.env.remove(&args[1].to_string()).is_none() {
        error!("Error: could not unset '{}'", args[1]);
        return Err(1);
    }

    Ok(0)
}

fn exec_with_config(cmd: &str, config: &mut Config) -> Result<usize, usize> {
    let mut cmd = cmd.to_string();

    // Replace `$key` with its value in the environment or an empty string
    let re = Regex::new("\\$\\w+");
    while let Some((a, b)) = re.find(&cmd) {
        let key: String = cmd.chars().skip(a + 1).take(b - a - 1).collect();
        let val = config.env.get(&key).map_or("", String::as_str);
        cmd = cmd.replace(&format!("${}", key), val);
    }

    let mut args = split_args(&cmd);

    // Replace command alias
    if let Some(alias) = config.aliases.get(&args[0]) {
        args.remove(0);
        for arg in alias.split(' ').rev() {
            args.insert(0, arg.to_string())
        }
    }

    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

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

        if Regex::new("^<=+$").is_match(args[i]) { // Redirect input stream
            is_fat_arrow = true;
            left_handle = 0;
        } else if Regex::new("^\\d*=+>$").is_match(args[i]) { // Redirect output stream(s)
            is_fat_arrow = true;
            left_handle = 1;
        } else if Regex::new("^\\d*-*>\\d*$").is_match(args[i]) { // Pipe output stream(s)
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
                return Err(1);
            }
            let path = args[i + 1];
            if api::fs::reopen(path, left_handle).is_err() {
                println!("Could not open path for redirection");
                return Err(1);
            }
            args.remove(i); // Remove redirection from args
            args.remove(i); // Remove path from args
            n -= 2;
        } else if is_thin_arrow { // TODO: Implement pipes
            println!("Could not parse arrow");
            return Err(1);
        }
    }

    let res = match args[0] {
        ""         => Ok(0),
        "2048"     => usr::pow::main(&args),
        "alias"    => cmd_alias(&args, config),
        "base64"   => usr::base64::main(&args),
        "beep"     => usr::beep::main(&args),
        "calc"     => usr::calc::main(&args),
        "chess"    => usr::chess::main(&args),
        "colors"   => usr::colors::main(&args),
        "copy"     => usr::copy::main(&args),
        "date"     => usr::date::main(&args),
        "delete"   => usr::delete::main(&args),
        "dhcp"     => usr::dhcp::main(&args),
        "disk"     => usr::disk::main(&args),
        "edit"     => usr::editor::main(&args),
        "elf"      => usr::elf::main(&args),
        "env"      => usr::env::main(&args),
        "find"     => usr::find::main(&args),
        "geotime"  => usr::geotime::main(&args),
        "goto"     => cmd_change_dir(&args, config),
        "help"     => usr::help::main(&args),
        "hex"      => usr::hex::main(&args),
        "host"     => usr::host::main(&args),
        "http"     => usr::http::main(&args),
        "httpd"    => usr::httpd::main(&args),
        "install"  => usr::install::main(&args),
        "keyboard" => usr::keyboard::main(&args),
        "lisp"     => usr::lisp::main(&args),
        "list"     => usr::list::main(&args),
        "memory"   => usr::memory::main(&args),
        "move"     => usr::r#move::main(&args),
        "net"      => usr::net::main(&args),
        "pci"      => usr::pci::main(&args),
        "proc"     => cmd_proc(&args),
        "quit"     => Err(255),
        "read"     => usr::read::main(&args),
        "set"      => cmd_set(&args, config),
        "shell"    => usr::shell::main(&args),
        "socket"   => usr::socket::main(&args),
        "tcp"      => usr::tcp::main(&args),
        "time"     => usr::time::main(&args),
        "unalias"  => cmd_unalias(&args, config),
        "unset"    => cmd_unset(&args, config),
        "user"     => usr::user::main(&args),
        "vga"      => usr::vga::main(&args),
        "write"    => usr::write::main(&args),
        _          => {
            let mut path = fs::realpath(args[0]);
            if path.len() > 1 {
                path = path.trim_end_matches('/').into();
            }
            match syscall::info(&path).map(|info| info.kind()) {
                Some(FileType::Dir) => {
                    sys::process::set_dir(&path);
                    config.env.insert("DIR".to_string(), sys::process::dir());
                    Ok(0)
                }
                Some(FileType::File) => {
                    spawn(&path, &args)
                }
                _ => {
                    let path = format!("/bin/{}", args[0]);
                    spawn(&path, &args)
                }
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

fn spawn(path: &str, args: &[&str]) -> Result<usize, usize> {
    match api::process::spawn(&path, &args) {
        Ok(0) => {
            Ok(0)
        }
        Ok(i) => {
            Err(1)
        }
        Err(_) => {
            error!("Could not execute '{}'", args[0]);
            Err(1)
        }
    }
}

fn repl(config: &mut Config) -> Result<usize, usize> {
    println!();

    let mut prompt = Prompt::new();
    let history_file = "~/.shell-history";
    prompt.history.load(history_file);
    prompt.completion.set(&shell_completer);

    let mut success = true;
    while let Some(cmd) = prompt.input(&prompt_string(success)) {
        let code = match exec_with_config(&cmd, config) {
            Err(255) => break,
            Ok(i) => i as isize,
            Err(i) => -(i as isize),
        };
        success = code.is_positive();
        config.env.insert("status".to_string(), format!("{}", code));
        prompt.history.add(&cmd);
        prompt.history.save(history_file);
        sys::console::drain();
        println!();
    }
    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move cursor to top
    Ok(0)
}

pub fn exec(cmd: &str) -> Result<usize, usize> {
    let mut config = Config::new();
    exec_with_config(cmd, &mut config)
}

pub fn main(args: &[&str]) -> Result<usize, usize> {
    let mut config = Config::new();

    if let Ok(rc) = fs::read_to_string("/ini/shell.sh") {
        for cmd in rc.split('\n') {
            exec_with_config(cmd, &mut config);
        }
    }

    if args.len() < 2 {
        config.env.insert(0.to_string(), args[0].to_string());

        repl(&mut config)
    } else {
        config.env.insert(0.to_string(), args[1].to_string());

        // Add script arguments to the environment as `$1`, `$2`, `$3`, ...
        for (i, arg) in args[2..].iter().enumerate() {
            config.env.insert((i + 1).to_string(), arg.to_string());
        }

        let pathname = args[1];
        if let Ok(contents) = api::fs::read_to_string(pathname) {
            for line in contents.split('\n') {
                if !line.is_empty() {
                    exec_with_config(line, &mut config);
                }
            }
            Ok(0)
        } else {
            println!("File not found '{}'", pathname);
            Err(1)
        }
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

    let mut config = Config::new();
    exec_with_config("set b 42", &mut config);
    exec_with_config("print a $b $c d => /test", &mut config);
    assert_eq!(api::fs::read_to_string("/test"), Ok("a 42 d\n".to_string()));

    sys::fs::dismount();
}

#[test_case]
fn test_glob_to_regex() {
    assert_eq!(glob_to_regex("hello.txt"), "^hello\\.txt$");
    assert_eq!(glob_to_regex("h?llo.txt"), "^h.llo\\.txt$");
    assert_eq!(glob_to_regex("h*.txt"), "^h.*\\.txt$");
    assert_eq!(glob_to_regex("*.txt"), "^.*\\.txt$");
    assert_eq!(glob_to_regex("\\w*.txt"), "^\\\\w.*\\.txt$");
}
