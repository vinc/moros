use super::{Config, Function};
use super::expand;
use super::parse::{parse, Chain, Exp, Op};
use super::primitive;

use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::process;
use crate::api::regex::Regex;
use crate::api::syscall;
use crate::sys::fs::FileType;
use crate::{api, usr};

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{fence, Ordering};

pub const AUTOCOMPLETE_COMMANDS: [&str; 47] = [
    "2048", "brainfuck", "calc", "chess", "copy", "date", "decode", "deflate",
    "dhcp", "diff", "disk", "draw", "drop", "edit", "elf", "encode", "false",
    "goto", "hash", "help", "hex", "host", "http", "httpd", "inflate",
    "install", "life", "lisp", "list", "memory", "mines", "move", "net", "pci",
    "print", "quit", "read", "render", "shell", "socket", "spell", "tcp",
    "time", "true", "user", "view", "write"
];

fn dispatch(args: &[&str], config: &mut Config) -> Result<(), ExitCode> {
    if let Some(function) = config.functions.get(args[0]) {
        return exec_function(function, &args[1..], config);
    }
    match args[0] {
        ""          => Ok(()),
        "2048"      => usr::pow::main(args),
        "alias"     => primitive::alias(args, config),
        "brainfuck" => usr::brainfuck::main(args),
        "calc"      => usr::calc::main(args),
        "chess"     => usr::chess::main(args),
        "copy"      => usr::copy::main(args),
        "date"      => usr::date::main(args),
        "decode"    => usr::decode::main(args),
        "deflate"   => usr::deflate::main(args),
        "dhcp"      => usr::dhcp::main(args),
        "diff"      => usr::diff::main(args),
        "disk"      => usr::disk::main(args),
        "draw"      => usr::draw::main(args),
        "drop"      => usr::drop::main(args),
        "edit"      => usr::edit::main(args),
        "elf"       => usr::elf::main(args),
        "encode"    => usr::encode::main(args),
        "find"      => usr::find::main(args),
        "goto"      => primitive::change_dir(args, config), // TODO: Remove this
        "hash"      => usr::hash::main(args),
        "help"      => usr::help::main(args),
        "hex"       => usr::hex::main(args),
        "host"      => usr::host::main(args),
        "http"      => usr::http::main(args),
        "httpd"     => usr::httpd::main(args),
        "inflate"   => usr::inflate::main(args),
        "install"   => usr::install::main(args),
        "life"      => usr::life::main(args),
        "lisp"      => usr::lisp::main(args),
        "list"      => usr::list::main(args),
        "logs"      => primitive::logs(),
        "memory"    => usr::memory::main(args),
        "mines"     => usr::mines::main(args),
        "move"      => usr::r#move::main(args),
        "net"       => usr::net::main(args),
        "pci"       => usr::pci::main(args),
        "pi"        => usr::pi::main(args),
        "print"     => primitive::print(args, config),
        "quit"      => Err(ExitCode::ShellExit),
        "read"      => usr::read::main(args),
        "render"    => usr::render::main(args),
        "set"       => primitive::set(args, config),
        "shell"     => usr::shell::main(args),
        "socket"    => usr::socket::main(args),
        "spell"     => usr::spell::main(args),
        "tcp"       => usr::tcp::main(args),
        "time"      => usr::time::main(args),
        "unalias"   => primitive::unalias(args, config),
        "unset"     => primitive::unset(args, config),
        "version"   => primitive::version(),
        "user"      => usr::user::main(args),
        "view"      => usr::view::main(args),
        "write"     => usr::write::main(args),
        "true"      => Ok(()),
        "false"     => Err(ExitCode::Failure),
        "panic"     => panic!("{}", args[1..].join(" ")),
        _ => {
            let mut path = fs::realpath(args[0]);
            if path.len() > 1 {
                path = path.trim_end_matches('/').into();
            }
            match syscall::info(&path).map(|info| info.kind()) {
                Some(FileType::Dir) => {
                    process::set_dir(&path);
                    config.env.insert("dir".to_string(), process::dir());
                    Ok(())
                }
                Some(FileType::File) => {
                    spawn(&path, args, config)
                }
                _ => {
                    let path = format!("/bin/{}", args[0]);
                    spawn(&path, args, config)
                }
            }
        }
    }
}

fn spawn(
    path: &str,
    args: &[&str],
    config: &mut Config
) -> Result<(), ExitCode> {
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
        Err(ExitCode::OpenError) => {
            error!("Could not open {:?}", args[0]);
            Err(ExitCode::OpenError)
        }
        Err(ExitCode::ReadError) => {
            error!("Could not read {:?}", args[0]);
            Err(ExitCode::ReadError)
        }
        Err(ExitCode::ExecError) => {
            error!("Could not execute {:?}", args[0]);
            Err(ExitCode::ExecError)
        }
        res => res,
    }
}

fn set_status(res: Result<(), ExitCode>, config: &mut Config) {
    let code = res.err().unwrap_or(ExitCode::Success);
    config.env.insert("status".to_string(), format!("{}", code as u8));
}

pub fn exec_command(cmd: &str, config: &mut Config) -> Result<(), ExitCode> {
    let cmd = expand::variables_expansion(cmd, config);
    let mut args = expand::split_args(cmd.trim(), false);
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
        } else if Regex::new("^<=*>+$").is_match(args[i]) {
            is_fat_arrow = true;
            left_handle = 0;
            if i + 1 < n {
                n += 2;
                args.insert(i + 2, args[i + 1]);
                args.insert(i + 2, args[i].trim_start_matches('<'));
                args[i] = "<=";
            }
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

fn exec_chain(chain: &Chain, config: &mut Config) -> Result<(), ExitCode> {
    let mut res = exec_command(chain.head, config);
    for &(op, cmd) in &chain.tail {
        if res == Err(ExitCode::ShellExit) {
            break;
        }
        set_status(res, config);
        res = match op {
            Op::And if res.is_ok() => exec_command(cmd, config),
            Op::Or if res.is_err() => exec_command(cmd, config),
            _ => res,
        };
    }
    res
}

fn exec_exp(exp: &Exp, config: &mut Config) -> Result<(), ExitCode> {
    match exp {
        Exp::Chain(chain) => exec_chain(chain, config),
        Exp::If(cond, then, other) => {
            let res = exec_chain(cond, config);
            if res == Err(ExitCode::ShellExit) {
                return res;
            }
            set_status(res, config);
            exec_body(if res.is_ok() { then } else { other }, config)
        }
        Exp::For(var, list, body) => {
            let list = expand::variables_expansion(list, config);
            let mut res = Ok(());
            for item in expand::split_args(&list, true) {
                config.env.insert(var.to_string(), item);
                res = exec_body(body, config);
                if res == Err(ExitCode::ShellExit) {
                    break;
                }
            }
            res
        }
        Exp::Def(name, params, body) => {
            let params = params.iter().map(|p| p.to_string()).collect();
            let body = body.to_string();
            let function = Function { params, body };
            config.functions.insert(name.to_string(), function);
            Ok(())
        }
    }
}

fn exec_body(body: &[Exp], config: &mut Config) -> Result<(), ExitCode> {
    let mut res = Ok(());
    for exp in body {
        res = exec_exp(&exp, config);
        if res == Err(ExitCode::ShellExit) {
            break;
        }
        set_status(res, config);
    }
    res
}

fn set_var(key: &str, val: Option<&str>, config: &mut Config) {
    match val {
        Some(val) => config.env.insert(key.to_string(), val.to_string()),
        None => config.env.remove(key),
    };
}

fn exec_function(
    function: &Function,
    args: &[&str],
    config: &Config
) -> Result<(), ExitCode> {
    let mut config = config.clone();
    for (i, param) in function.params.iter().enumerate() {
        set_var(param, args.get(i).copied(), &mut config);
    }
    for i in 1..10 {
        set_var(&i.to_string(), args.get(i - 1).copied(), &mut config);
    }
    exec_script(&function.body, &mut config)
}

pub fn exec_script(script: &str, config: &mut Config) -> Result<(), ExitCode> {
    match parse(script) {
        Ok((_, body)) => exec_body(&body, config),
        Err(_) => {
            error!("Could not parse script");
            let res = Err(ExitCode::Failure);
            set_status(res, config);
            res
        }
    }
}
