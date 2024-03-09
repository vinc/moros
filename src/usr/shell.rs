use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::prompt::Prompt;
use crate::api::regex::Regex;
use crate::api::syscall;
use crate::sys::fs::FileType;
use crate::{api, sys, usr};

use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{fence, Ordering};

// TODO: Scan /bin
const AUTOCOMPLETE_COMMANDS: [&str; 36] = [
    "2048", "base64", "calc", "copy", "date", "delete", "dhcp", "disk", "edit",
    "elf", "env", "goto", "hash", "help", "hex", "host", "http", "httpd",
    "install", "keyboard", "life", "lisp", "list", "memory", "move", "net",
    "pci", "quit", "read", "shell", "socket", "tcp", "time", "user", "vga",
    "write",
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
            // Copy the process environment to the shell environment
            env.insert(key, val);
        }
        env.insert("DIR".to_string(), sys::process::dir());
        env.insert("status".to_string(), "0".to_string());
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
    let mut args = split_args(line);
    if line.ends_with(' ') {
        args.push(String::new());
    }
    let i = args.len() - 1;

    // Autocomplete command
    if i == 0 && !args[i].starts_with('/') && !args[i].starts_with('~') {
        for cmd in autocomplete_commands() {
            if let Some(entry) = cmd.strip_prefix(&args[i]) {
                entries.push(entry.into());
            }
        }
    }

    // Autocomplete path
    let pathname = fs::realpath(&args[i]);
    let dirname = fs::dirname(&pathname);
    let filename = fs::filename(&pathname);
    let sep = if dirname.ends_with('/') { "" } else { "/" };
    if let Ok(files) = fs::read_dir(dirname) {
        for file in files {
            let name = file.name();
            if name.starts_with(filename) {
                if args.len() == 1 && !file.is_dir() {
                    continue;
                }
                let end = if args.len() != 1 && file.is_dir() {
                    "/"
                } else {
                    ""
                };
                let path = format!("{}{}{}{}", dirname, sep, name, end);
                entries.push(path[pathname.len()..].into());
            }
        }
    }

    entries.sort();
    entries
}

pub fn prompt_string(success: bool) -> String {
    let csi_line1 = Style::color("Blue");
    let csi_line2 = Style::color("Magenta");
    let csi_error = Style::color("Red");
    let csi_reset = Style::reset();

    let mut current_dir = sys::process::dir();
    if let Some(home) = sys::process::env("HOME") {
        if current_dir.starts_with(&home) {
            let n = home.len();
            current_dir.replace_range(..n, "~");
        }
    }
    let line1 = format!("{}{}{}", csi_line1, current_dir, csi_reset);
    let line2 = format!(
        "{}>{} ",
        if success { csi_line2 } else { csi_error },
        csi_reset
    );
    format!("{}\n{}", line1, line2)
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
    format!(
        "^{}$",
        pattern.replace('\\', "\\\\") // `\` string literal
               .replace('.', "\\.") // `.` string literal
               .replace('*', ".*") // `*` match zero or more chars except `/`
               .replace('?', ".") // `?` match any char except `/`
    )
}

fn glob(arg: &str) -> Vec<String> {
    let mut matches = Vec::new();
    if is_globbing(arg) {
        let (dir, pattern, show_dir) = if arg.contains('/') {
            let d = fs::dirname(arg).to_string();
            let n = fs::filename(arg).to_string();
            (d, n, true)
        } else {
            (sys::process::dir(), arg.to_string(), false)
        };
        let re = Regex::new(&glob_to_regex(&pattern));
        let sep = if dir == "/" { "" } else { "/" };
        if let Ok(files) = fs::read_dir(&dir) {
            for file in files {
                let name = file.name();
                if re.is_match(&name) {
                    if show_dir {
                        matches.push(format!("{}{}{}", dir, sep, name));
                    } else {
                        matches.push(name);
                    }
                }
            }
        }
    } else {
        matches.push(arg.to_string());
    }
    matches
}

pub fn parse_str(s: &str) -> String {
    let mut res = String::new();
    let mut is_escaped = false;
    for c in s.chars() {
        match c {
            '\\' if !is_escaped => {
                is_escaped = true;
                continue;
            }
            _ if !is_escaped => res.push(c),
            '\\' => res.push(c),
            '"' => res.push(c),
            'n' => res.push('\n'),
            'r' => res.push('\r'),
            't' => res.push('\t'),
            'b' => res.push('\x08'),
            'e' => res.push('\x1B'),
            _ => {}
        }
        is_escaped = false;
    }
    res
}

pub fn split_args(cmd: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut i = 0;
    let mut n = cmd.len();
    let mut is_quote = false;
    let mut is_escaped = false;

    for (j, c) in cmd.char_indices() {
        if c == '#' && !is_quote {
            n = j; // Discard comments
            break;
        } else if c == ' ' && !is_quote {
            if i != j && !cmd[i..j].trim().is_empty() {
                if args.is_empty() {
                    args.push(cmd[i..j].to_string()) // program name
                } else {
                    args.extend(glob(&cmd[i..j])) // program args
                }
            }
            i = j + 1;
        } else if c == '"' && !is_escaped {
            is_quote = !is_quote;
            if !is_quote {
                args.push(parse_str(&cmd[i..j]));
            }
            i = j + 1;
        }
        if c == '\\' && !is_escaped {
            is_escaped = true;
        } else {
            is_escaped = false;
        }
    }

    if i < n {
        if is_quote {
            n -= 1;
            args.push(cmd[i..n].to_string());
        } else if args.is_empty() {
            args.push(cmd[i..n].to_string());
        } else if !cmd[i..n].trim().is_empty() {
            args.extend(glob(&cmd[i..n]))
        }
    }

    if n == 0 {
        args.push("".to_string());
    }

    args.iter().map(|s| tilde_expansion(s)).collect()
}

// Replace `~` with the value of `$HOME` when it's at the begining of an arg
fn tilde_expansion(arg: &str) -> String {
    if let Some(home) = sys::process::env("HOME") {
        let tilde = "~";
        if arg == tilde || arg.starts_with("~/") {
            return arg.replacen(tilde, &home, 1);
        }
    }
    arg.to_string()
}

fn variables_expansion(cmd: &str, config: &mut Config) -> String {
    let mut cmd = cmd.to_string();

    // Special cases for none alphanum (\w) variables
    cmd = cmd.replace("$?", "$status");
    cmd = cmd.replace("$*", "$1 $2 $3 $4 $5 $6 $7 $8 $9");

    // Replace alphanum `$key` with its value in the environment
    // or an empty string.
    let re = Regex::new("\\$\\w+");
    while let Some((a, b)) = re.find(&cmd) {
        let key: String = cmd.chars().skip(a + 1).take(b - a - 1).collect();
        let val = config.env.get(&key).map_or("", String::as_str);
        cmd = cmd.replace(&format!("${}", key), val);
    }

    cmd
}

fn cmd_change_dir(args: &[&str], config: &mut Config) -> Result<(), ExitCode> {
    match args.len() {
        1 => {
            println!("{}", sys::process::dir());
            Ok(())
        }
        2 => {
            let mut path = fs::realpath(args[1]);
            if path.len() > 1 {
                path = path.trim_end_matches('/').into();
            }
            if api::fs::is_dir(&path) {
                sys::process::set_dir(&path);
                config.env.insert("DIR".to_string(), sys::process::dir());
                Ok(())
            } else {
                error!("Could not find file '{}'", path);
                Err(ExitCode::Failure)
            }
        }
        _ => Err(ExitCode::Failure),
    }
}

fn cmd_alias(args: &[&str], config: &mut Config) -> Result<(), ExitCode> {
    if args.len() != 3 {
        let csi_option = Style::color("LightCyan");
        let csi_title = Style::color("Yellow");
        let csi_reset = Style::reset();
        eprintln!(
            "{}Usage:{} alias {}<key> <val>{1}",
            csi_title, csi_reset, csi_option
        );
        return Err(ExitCode::UsageError);
    }
    config.aliases.insert(args[1].to_string(), args[2].to_string());
    Ok(())
}

fn cmd_unalias(args: &[&str], config: &mut Config) -> Result<(), ExitCode> {
    if args.len() != 2 {
        let csi_option = Style::color("LightCyan");
        let csi_title = Style::color("Yellow");
        let csi_reset = Style::reset();
        eprintln!(
            "{}Usage:{} unalias {}<key>{1}",
            csi_title, csi_reset, csi_option
        );
        return Err(ExitCode::UsageError);
    }

    if config.aliases.remove(&args[1].to_string()).is_none() {
        error!("Could not unalias '{}'", args[1]);
        return Err(ExitCode::Failure);
    }

    Ok(())
}

fn cmd_set(args: &[&str], config: &mut Config) -> Result<(), ExitCode> {
    if args.len() != 3 {
        let csi_option = Style::color("LightCyan");
        let csi_title = Style::color("Yellow");
        let csi_reset = Style::reset();
        eprintln!(
            "{}Usage:{} set {}<key> <val>{1}",
            csi_title, csi_reset, csi_option
        );
        return Err(ExitCode::UsageError);
    }

    config.env.insert(args[1].to_string(), args[2].to_string());
    Ok(())
}

fn cmd_unset(args: &[&str], config: &mut Config) -> Result<(), ExitCode> {
    if args.len() != 2 {
        let csi_option = Style::color("LightCyan");
        let csi_title = Style::color("Yellow");
        let csi_reset = Style::reset();
        eprintln!(
            "{}Usage:{} unset {}<key>{1}",
            csi_title, csi_reset, csi_option
        );
        return Err(ExitCode::UsageError);
    }

    if config.env.remove(&args[1].to_string()).is_none() {
        error!("Could not unset '{}'", args[1]);
        return Err(ExitCode::Failure);
    }

    Ok(())
}

fn cmd_logs() -> Result<(), ExitCode> {
    print!("{}", sys::log::read());
    Ok(())
}

fn cmd_version() -> Result<(), ExitCode> {
    println!(
        "MOROS v{}",
        option_env!("MOROS_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))
    );
    Ok(())
}

fn exec_with_config(cmd: &str, config: &mut Config) -> Result<(), ExitCode> {
    let cmd = variables_expansion(cmd, config);
    let mut args = split_args(cmd.trim());
    if args.is_empty() {
        return Ok(());
    }

    // Replace command alias
    if let Some(alias) = config.aliases.get(&args[0]) {
        args.remove(0);
        for arg in alias.split(' ').rev() {
            args.insert(0, arg.to_string())
        }
    }

    let mut args: Vec<&str> = args.iter().map(String::as_str).collect();

    // Redirections
    let mut restore_handles = false;
    let mut n = args.len();
    let mut i = 0;
    loop {
        if i == n {
            break;
        }

        let mut is_fat_arrow = false;
        let mut is_thin_arrow = false;
        let mut head_count = 0;
        let mut left_handle;
        if Regex::new("^[?\\d*]?-+>$").is_match(args[i]) {
            // Pipes
            // read foo.txt --> write bar.txt
            // read foo.txt -> write bar.txt
            // read foo.txt [2]-> write /dev/null
            is_thin_arrow = true;
            left_handle = 1;
        } else if Regex::new("^[?\\d*]?=*>+[?\\d*]?$").is_match(args[i]) {
            // Redirections to
            // read foo.txt ==> bar.txt
            // read foo.txt => bar.txt
            // read foo.txt > bar.txt
            // read foo.txt [1]=> /dev/null
            // read foo.txt [1]=>[3]
            is_fat_arrow = true;
            left_handle = 1;
        } else if Regex::new("^<=*$").is_match(args[i]) {
            // Redirections from
            // write bar.txt <== foo.txt
            // write bar.txt <= foo.txt
            // write bar.txt < foo.txt
            is_fat_arrow = true;
            left_handle = 0;
        } else {
            i += 1;
            continue;
        }

        // Parse handles
        let mut num = String::new();
        for c in args[i].chars() {
            match c {
                '[' | ']' | '-' | '=' => {
                    continue;
                }
                '<' | '>' => {
                    head_count += 1;
                    if let Ok(handle) = num.parse() {
                        left_handle = handle;
                    }
                    num.clear();
                }
                _ => {
                    num.push(c);
                }
            }
        }

        if is_fat_arrow {
            // Redirections
            restore_handles = true;
            if !num.is_empty() {
                // if let Ok(right_handle) = num.parse() {}
                error!("Redirecting to a handle has not been implemented yet");
                return Err(ExitCode::Failure);
            } else {
                if i == n - 1 {
                    error!("Could not parse path for redirection");
                    return Err(ExitCode::Failure);
                }
                let path = args[i + 1];
                let append_mode = head_count > 1;
                if api::fs::reopen(path, left_handle, append_mode).is_err() {
                    error!("Could not open path for redirection");
                    return Err(ExitCode::Failure);
                }
                args.remove(i); // Remove path from args
                n -= 1;
            }
            n -= 1;
            args.remove(i); // Remove redirection from args
        } else if is_thin_arrow {
            error!("Piping has not been implemented yet");
            return Err(ExitCode::Failure);
        }
    }

    fence(Ordering::SeqCst);
    let res = dispatch(&args, config);

    // TODO: Remove this when redirections are done in spawned process
    if restore_handles {
        for i in 0..3 {
            api::fs::reopen("/dev/console", i, false).ok();
        }
    }

    res
}

fn dispatch(args: &[&str], config: &mut Config) -> Result<(), ExitCode> {
    match args[0] {
        ""         => Ok(()),
        "2048"     => usr::pow::main(args),
        "alias"    => cmd_alias(args, config),
        "base64"   => usr::base64::main(args),
        "beep"     => usr::beep::main(args),
        "calc"     => usr::calc::main(args),
        "chess"    => usr::chess::main(args),
        "copy"     => usr::copy::main(args),
        "date"     => usr::date::main(args),
        "delete"   => usr::delete::main(args),
        "dhcp"     => usr::dhcp::main(args),
        "disk"     => usr::disk::main(args),
        "edit"     => usr::editor::main(args),
        "elf"      => usr::elf::main(args),
        "env"      => usr::env::main(args),
        "find"     => usr::find::main(args),
        "goto"     => cmd_change_dir(args, config), // TODO: Remove this
        "hash"     => usr::hash::main(args),
        "help"     => usr::help::main(args),
        "hex"      => usr::hex::main(args),
        "host"     => usr::host::main(args),
        "http"     => usr::http::main(args),
        "httpd"    => usr::httpd::main(args),
        "install"  => usr::install::main(args),
        "keyboard" => usr::keyboard::main(args),
        "life"     => usr::life::main(args),
        "lisp"     => usr::lisp::main(args),
        "list"     => usr::list::main(args),
        "logs"     => cmd_logs(),
        "memory"   => usr::memory::main(args),
        "move"     => usr::r#move::main(args),
        "net"      => usr::net::main(args),
        "pci"      => usr::pci::main(args),
        "pi"       => usr::pi::main(args),
        "quit"     => Err(ExitCode::ShellExit),
        "read"     => usr::read::main(args),
        "set"      => cmd_set(args, config),
        "shell"    => usr::shell::main(args),
        "socket"   => usr::socket::main(args),
        "tcp"      => usr::tcp::main(args),
        "time"     => usr::time::main(args),
        "unalias"  => cmd_unalias(args, config),
        "unset"    => cmd_unset(args, config),
        "version"  => cmd_version(),
        "user"     => usr::user::main(args),
        "vga"      => usr::vga::main(args),
        "write"    => usr::write::main(args),
        "panic"    => panic!("{}", args[1..].join(" ")),
        _ => {
            let mut path = fs::realpath(args[0]);
            if path.len() > 1 {
                path = path.trim_end_matches('/').into();
            }
            match syscall::info(&path).map(|info| info.kind()) {
                Some(FileType::Dir) => {
                    sys::process::set_dir(&path);
                    config.env.insert("DIR".to_string(), sys::process::dir());
                    Ok(())
                }
                Some(FileType::File) => spawn(&path, args, config),
                _ => {
                    let path = format!("/bin/{}", args[0]);
                    spawn(&path, args, config)
                }
            }
        }
    }
}

fn spawn(path: &str, args: &[&str], config: &mut Config) -> Result<(), ExitCode> {
    // Script
    if let Ok(contents) = fs::read_to_string(path) {
        if contents.starts_with("#!") {
            if let Some(line) = contents.lines().next() {
                let mut new_args = Vec::with_capacity(args.len() + 1);
                new_args.push(line[2..].trim());
                new_args.push(path);
                new_args.extend(&args[1..]);
                return dispatch(&new_args, config);
            }
        }
    }

    // Binary
    match api::process::spawn(path, args) {
        Err(ExitCode::ExecError) => {
            error!("Could not execute '{}'", args[0]);
            Err(ExitCode::ExecError)
        }
        Err(ExitCode::ReadError) => {
            error!("Could not read '{}'", args[0]);
            Err(ExitCode::ReadError)
        }
        Err(ExitCode::OpenError) => {
            error!("Could not open '{}'", args[0]);
            Err(ExitCode::OpenError)
        }
        res => res,
    }
}

fn repl(config: &mut Config) -> Result<(), ExitCode> {
    println!();

    let mut prompt = Prompt::new();
    let history_file = "~/.shell-history";
    prompt.history.load(history_file);
    prompt.completion.set(&shell_completer);

    let mut code = ExitCode::Success;
    let success = code;
    while let Some(cmd) = prompt.input(&prompt_string(code == success)) {
        code = match exec_with_config(&cmd, config) {
            Err(ExitCode::ShellExit) => break,
            Err(e) => e,
            Ok(()) => ExitCode::Success,
        };
        config.env.insert("status".to_string(), format!("{}", code as u8));
        prompt.history.add(&cmd);
        prompt.history.save(history_file);
        sys::console::drain();
        println!();
    }
    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
    Ok(())
}

pub fn exec(cmd: &str) -> Result<(), ExitCode> {
    let mut config = Config::new();
    exec_with_config(cmd, &mut config)
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut config = Config::new();

    if let Ok(rc) = fs::read_to_string("/ini/shell.sh") {
        for cmd in rc.split('\n') {
            exec_with_config(cmd, &mut config).ok();
        }
    }

    if args.len() < 2 {
        config.env.insert(0.to_string(), args[0].to_string());

        repl(&mut config)
    } else {
        if args[1] == "-h" || args[1] == "--help" {
            return help();
        }
        config.env.insert(0.to_string(), args[1].to_string());

        // Add script arguments to the environment as `$1`, `$2`, `$3`, ...
        for (i, arg) in args[2..].iter().enumerate() {
            config.env.insert((i + 1).to_string(), arg.to_string());
        }

        let path = args[1];
        if let Ok(contents) = api::fs::read_to_string(path) {
            for line in contents.split('\n') {
                if !line.is_empty() {
                    exec_with_config(line, &mut config).ok();
                }
            }
            Ok(())
        } else {
            error!("Could not find file '{}'", path);
            Err(ExitCode::Failure)
        }
    }
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} shell {}[<file> [<args>]]{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    Ok(())
}

#[test_case]
fn test_shell() {
    use alloc::string::ToString;

    sys::fs::mount_mem();
    sys::fs::format_mem();
    usr::install::copy_files(false);

    // Redirect standard output
    exec("print test1 => /tmp/test1").ok();
    assert_eq!(
        api::fs::read_to_string("/tmp/test1"),
        Ok("test1\n".to_string())
    );

    // Redirect standard output explicitely
    exec("print test2 1=> /tmp/test2").ok();
    assert_eq!(
        api::fs::read_to_string("/tmp/test2"),
        Ok("test2\n".to_string())
    );

    // Redirect standard error explicitely
    exec("hex /nope 2=> /tmp/test3").ok();
    assert!(api::fs::read_to_string("/tmp/test3").unwrap().
        contains("Could not find file '/nope'"));

    let mut config = Config::new();
    exec_with_config("set b 42", &mut config).ok();
    exec_with_config("print a $b $c d => /test", &mut config).ok();
    assert_eq!(api::fs::read_to_string("/test"), Ok("a 42 d\n".to_string()));

    sys::fs::dismount();
}

#[test_case]
fn test_split_args() {
    use alloc::vec;
    assert_eq!(split_args(""), vec![""]);
    assert_eq!(split_args("print"), vec!["print"]);
    assert_eq!(split_args("print "), vec!["print"]);
    assert_eq!(split_args("print  "), vec!["print"]);
    assert_eq!(split_args("print # comment"), vec!["print"]);
    assert_eq!(split_args("print foo"), vec!["print", "foo"]);
    assert_eq!(split_args("print foo "), vec!["print", "foo"]);
    assert_eq!(split_args("print foo  "), vec!["print", "foo"]);
    assert_eq!(split_args("print foo # comment"), vec!["print", "foo"]);
    assert_eq!(split_args("print foo bar"), vec!["print", "foo", "bar"]);
    assert_eq!(split_args("print foo   bar"), vec!["print", "foo", "bar"]);
    assert_eq!(split_args("print   foo   bar"), vec!["print", "foo", "bar"]);
    assert_eq!(split_args("print foo \"bar\""), vec!["print", "foo", "bar"]);
    assert_eq!(split_args("print foo \"\""), vec!["print", "foo", ""]);
    assert_eq!(
        split_args("print foo \"bar\" "),
        vec!["print", "foo", "bar"]
    );
    assert_eq!(split_args("print foo \"\" "), vec!["print", "foo", ""]);
}

#[test_case]
fn test_glob_to_regex() {
    assert_eq!(glob_to_regex("hello.txt"), "^hello\\.txt$");
    assert_eq!(glob_to_regex("h?llo.txt"), "^h.llo\\.txt$");
    assert_eq!(glob_to_regex("h*.txt"), "^h.*\\.txt$");
    assert_eq!(glob_to_regex("*.txt"), "^.*\\.txt$");
    assert_eq!(glob_to_regex("\\w*.txt"), "^\\\\w.*\\.txt$");
}

#[test_case]
fn test_variables_expansion() {
    let mut config = Config::new();
    exec_with_config("set foo 42", &mut config).ok();
    exec_with_config("set bar \"Alice and Bob\"", &mut config).ok();
    assert_eq!(variables_expansion("print $foo", &mut config), "print 42");
    assert_eq!(
        variables_expansion("print \"Hello $bar\"", &mut config),
        "print \"Hello Alice and Bob\""
    );
}
