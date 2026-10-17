use super::Config;

use crate::api::fs;
use crate::api::process;
use crate::api::regex::Regex;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

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

fn glob(arg: &str) -> Vec<String> {
    let mut matches = Vec::new();
    if is_globbing(arg) {
        let (dir, pattern, show_dir) = if arg.contains('/') {
            let d = fs::dirname(arg).to_string();
            let n = fs::filename(arg).to_string();
            (d, n, true)
        } else {
            (process::dir(), arg.to_string(), false)
        };
        let re = Regex::from_glob(&pattern);
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
    if let Some(home) = process::env_var("HOME") {
        let tilde = "~";
        if arg == tilde || arg.starts_with("~/") {
            return arg.replacen(tilde, &home, 1);
        }
    }
    arg.to_string()
}

pub fn variables_expansion(cmd: &str, config: &mut Config) -> String {
    let mut cmd = cmd.to_string();

    // Special cases for non alphanum (\w) variables
    cmd = cmd.replace("$?", "$status");
    cmd = cmd.replace("$*", "$1 $2 $3 $4 $5 $6 $7 $8 $9");

    // Replace alphanum `$key` with its value in the environment
    // or an empty string
    let re = Regex::new("\\$\\w+");
    while let Some((a, b)) = re.find(&cmd) {
        let key: String = cmd.chars().skip(a + 1).take(b - a - 1).collect();
        let val = config.env.get(&key).map_or("", String::as_str);
        cmd = cmd.replace(&format!("${}", key), val);
    }

    // Replace `$(command)` with its captured output
    let re = Regex::new("\\$(.*?)");
    while let Some((a, b)) = re.find(&cmd) {
        let sub: String = cmd.drain(a..b).skip(2).take(b - a - 3).collect();
        if let Ok(buf) = super::exec_to_bytes(&sub) {
            if let Ok(txt) = String::from_utf8(buf) {
                cmd.insert_str(a, txt.trim());
            }
        }
    }

    cmd
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
fn test_variables_expansion() {
    use super::exec::exec_command;

    let mut config = Config::new();
    exec_command("set foo 42", &mut config).ok();
    exec_command("set bar \"Alice and Bob\"", &mut config).ok();
    assert_eq!(variables_expansion("print $foo", &mut config), "print 42");
    assert_eq!(
        variables_expansion("print \"Hello $bar\"", &mut config),
        "print \"Hello Alice and Bob\""
    );
}
