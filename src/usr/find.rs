use alloc::string::ToString;
use alloc::vec::Vec;
use crate::{sys, usr};
use crate::api::fs;
use crate::api::regex::Regex;
use crate::api::console::Style;

// > find /tmp -name *.txt -line hello
pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let mut path: &str = &sys::process::dir();
    let mut name = None;
    let mut line = None;
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "--name" | "-n" => {
                if i + 1 < n {
                    name = Some(args[i + 1]);
                    i += 1;
                } else {
                    println!("Missing name");
                    return usr::shell::ExitCode::CommandError;
                }
            },
            "--line" | "-l" => {
                if i + 1 < n {
                    line = Some(args[i + 1]);
                    i += 1;
                } else {
                    println!("Missing line");
                    return usr::shell::ExitCode::CommandError;
                }
            },
            _ => path = args[i],
        }
        i += 1;
    }

    if let Some(pattern) = line {
        let re = Regex::new(pattern);
        if let Ok(lines) = fs::read_to_string(path) {
            let mut matches = Vec::new();
            for (i, line) in lines.split('\n').enumerate() {
                if re.is_match(line) {
                    matches.push((i, line));
                }
            }
            if !matches.is_empty() {
                // TODO: Print filename if we are walking a dir
                let width = matches[matches.len() - 1].0.to_string().len();
                for (i, line) in matches {
                    println!("{}{:>width$}:{} {}", Style::color("Yellow"), i, Style::reset(), line, width = width);
                }
            }
        }
    }

    usr::shell::ExitCode::CommandSuccessful
}
