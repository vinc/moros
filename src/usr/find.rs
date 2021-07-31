use crate::{sys, usr};
use crate::api::fs;
use crate::api::regex::Regex;
use crate::api::console::Style;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::iter::FromIterator;

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

    let num = Style::color("Yellow");
    let color = Style::color("Red");
    let reset = Style::reset();

    if let Some(pattern) = line {
        let re = Regex::new(pattern);
        if let Ok(lines) = fs::read_to_string(path) {
            let mut matches = Vec::new();
            for (i, line) in lines.split('\n').enumerate() {
                let line: Vec<char> = line.chars().collect();
                let mut l = String::new();
                let mut j = 0;
                while let Some((a, b)) = re.find(&String::from_iter(&line[j..])) {
                    let m = j + a;
                    let n = j + b;
                    let before = String::from_iter(&line[j..m]);
                    let matched = String::from_iter(&line[m..n]);
                    l = format!("{}{}{}{}{}", l, before, color, matched, reset);
                    j = n;
                }
                if !l.is_empty() {
                    let after = String::from_iter(&line[j..]);
                    l.push_str(&after);
                    matches.push((i + 1, l)); // 1-index line numbers
                }
            }
            if !matches.is_empty() {
                // TODO: Print filename if we are walking a dir
                let width = matches[matches.len() - 1].0.to_string().len();
                for (i, line) in matches {
                    println!("{}{:>width$}:{} {}", num, i, reset, line, width = width);
                }
            }
        }
    }

    usr::shell::ExitCode::CommandSuccessful
}
