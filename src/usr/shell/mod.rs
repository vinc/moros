mod exec;
mod expand;
mod parse;
mod primitive;

use exec::exec_script;

use crate::api::console::Style;
use crate::api::fs;
use crate::api::process;
use crate::api::process::ExitCode;
use crate::api::prompt::Prompt;
use crate::api::syscall;
use crate::{api, sys};

use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

#[derive(Clone)]
struct Function {
    params: Vec<String>,
    body: String
}

#[derive(Clone)]
struct Config {
    env: BTreeMap<String, String>,
    aliases: BTreeMap<String, String>,
    functions: BTreeMap<String, Function>
}

impl Config {
    fn new() -> Config {
        let functions = BTreeMap::new();
        let aliases = BTreeMap::new();
        let mut env = BTreeMap::new();
        for (key, val) in process::env() {
            // Copy the process environment to the shell environment
            env.insert(key, val);
        }
        env.insert("dir".to_string(), process::dir());
        env.insert("status".to_string(), "0".to_string());
        Config { env, aliases, functions }
    }
}

fn autocomplete_commands() -> Vec<String> {
    let mut res = Vec::new();
    for cmd in exec::AUTOCOMPLETE_COMMANDS {
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
    let mut args = expand::split_args(line, false);
    if args.is_empty() || line.ends_with(' ') {
        args.push(String::new());
    }
    let i = args.len() - 1;

    // Autocomplete command
    if i == 0 && !fs::is_absolute_path(&args[i]) {
        for cmd in autocomplete_commands() {
            if let Some(entry) = cmd.strip_prefix(&args[i]) {
                entries.push(entry.into());
            }
        }
    }

    // Autocomplete path
    if (i == 0 && fs::is_absolute_path(&args[i])) || i > 0 {
        let path = fs::realpath(&args[i]);
        let (dirname, filename) = if path.len() > 1 && path.ends_with('/') {
            // List files in dir (/path/to/ -> /path/to/file.txt)
            (path.trim_end_matches('/'), "")
        } else {
            // List matching files (/path/to/fi -> /path/to/file.txt)
            (fs::dirname(&path), fs::filename(&path))
        };
        let sep = if dirname.ends_with('/') { "" } else { "/" };
        if let Ok(files) = fs::read_dir(dirname) {
            for file in files {
                let name = file.name();
                if name.starts_with(filename) {
                    let end = if file.is_dir() { "/" } else { "" };
                    let entry = format!("{}{}{}{}", dirname, sep, name, end);
                    entries.push(entry[path.len()..].into());
                }
            }
        }
    }

    entries.sort();
    entries
}

pub fn prompt_string(success: bool) -> String {
    let csi_line1 = Style::color("navy");
    let csi_line2 = Style::color("purple");
    let csi_error = Style::color("maroon");
    let csi_reset = Style::reset();

    let mut current_dir = process::dir();
    if let Some(home) = process::env_var("HOME") {
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

fn repl(config: &mut Config) -> Result<(), ExitCode> {
    println!();

    let mut prompt = Prompt::new();
    let history_file = "~/.shell-history";
    prompt.history.load(history_file);
    prompt.completion.set(&shell_completer);

    let mut success = true;
    while let Some(cmd) = prompt.input(&prompt_string(success)) {
        let res = exec_script(&cmd, config);
        if res == Err(ExitCode::ShellExit) {
            break;
        }
        success = res.is_ok();
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
    exec_script(cmd, &mut config)
}

pub fn exec_to_bytes(cmd: &str) -> Result<Vec<u8>, ExitCode> {
    let pipe = "/dev/pipe";
    let stdout = 1;
    if let Some(info) = syscall::info(pipe) {
        if info.is_device() {
            if let Some(handle) = api::fs::open_device(pipe) {
                syscall::dup(handle, stdout).ok();
                exec(cmd)?;
                api::fs::reopen("/dev/console", stdout, false).ok();
                let n = info.size() as usize;
                let mut buf = vec![0; n];
                let mut res = Vec::new();
                while let Some(bytes) = syscall::read(handle, &mut buf) {
                    if bytes == 0 {
                        break;
                    }
                    res.extend_from_slice(&buf[..bytes]);
                }
                syscall::close(handle);
                return Ok(res);
            }
        }
    }
    error!("Could not open pipe");
    Err(ExitCode::Failure)
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut config = Config::new();

    if let Ok(buf) = fs::read_to_string("/ini/shell.sh") {
        exec_script(&buf, &mut config).ok();
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
        if let Ok(buf) = api::fs::read_to_string(path) {
            match exec_script(&buf, &mut config) {
                Err(ExitCode::ShellExit) => Ok(()),
                res => res,
            }
        } else {
            error!("Could not read file {:?}", path);
            Err(ExitCode::Failure)
        }
    }
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} shell {}[<file> [<args>]]{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    Ok(())
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
#[test_case]
fn test_shell() {
    use exec::exec_command;
    use crate::usr;
    use alloc::string::ToString;

    sys::fs::mount_mem();
    sys::fs::format_mem();
    usr::install::copy_files(false);

    // Split and glob all arguments
    let args = expand::split_args("/ini/*.sh", true);
    assert_eq!(args, vec!["/ini/boot.sh", "/ini/shell.sh"]);

    // Split and glob all arguments except the first
    let args = expand::split_args("/ini/*.sh", false);
    assert_eq!(args, vec!["/ini/*.sh"]);

    // Redirect standard output
    exec("echo test1 => /tmp/test1").ok();
    assert_eq!(
        api::fs::read_to_string("/tmp/test1"),
        Ok("test1\n".to_string())
    );

    // Redirect standard output explicitely
    exec("echo test2 1=> /tmp/test2").ok();
    assert_eq!(
        api::fs::read_to_string("/tmp/test2"),
        Ok("test2\n".to_string())
    );

    // Redirect standard error explicitely
    exec("hex /nope 2=> /tmp/test3").ok();
    assert!(api::fs::read_to_string("/tmp/test3").unwrap().
        contains("Could not read file \"/nope\""));

    let mut config = Config::new();
    exec_command("set b 42", &mut config).ok();
    exec_command("echo a $b $c d => /test", &mut config).ok();
    assert_eq!(api::fs::read_to_string("/test"), Ok("a 42 d\n".to_string()));

    sys::fs::dismount();
}
