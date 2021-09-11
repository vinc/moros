use crate::{sys, usr};
use crate::api::fs;
use crate::api::syscall;
use crate::api::regex::Regex;
use crate::api::console::Style;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::iter::FromIterator;

struct PrintingState {
    is_first_match: bool,
    is_recursive: bool,
}

impl PrintingState {
    fn new() -> Self {
        Self {
            is_first_match: true,
            is_recursive: false,
        }
    }
}

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

    if name.is_some() {
        todo!();
    }

    let mut state = PrintingState::new();
    if let Some(pattern) = line {
        print_matching_lines(path, pattern, &mut state);
    }

    usr::shell::ExitCode::CommandSuccessful
}

fn print_matching_lines(path: &str, pattern: &str, state: &mut PrintingState) {
    if let Some(dir) = sys::fs::Dir::open(path) {
        state.is_recursive = true;
        for file in dir.entries() {
            let file_path = format!("{}/{}", path, file.name());
            if file.is_dir() {
                print_matching_lines(&file_path, pattern, state);
            } else {
                print_matching_lines_in_file(&file_path, pattern, state);
            }
        }
    } else if syscall::stat(path).is_some() {
        print_matching_lines_in_file(&path, pattern, state);
    }
}

fn print_matching_lines_in_file(path: &str, pattern: &str, state: &mut PrintingState) {
    let name_color = Style::color("Cyan");
    let line_color = Style::color("Yellow");
    let match_color = Style::color("LightRed");
    let reset = Style::reset();

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
                l = format!("{}{}{}{}{}", l, before, match_color, matched, reset);
                j = n;
                if m == n || n >= line.len() {
                    // Some patterns like "" or ".*?" would never move the
                    // cursor on the line and some like ".*" would match the
                    // whole line at once. In both cases we print the line,
                    // and we color it in the latter case.
                    break;
                }
            }
            if !l.is_empty() {
                let after = String::from_iter(&line[j..]);
                l.push_str(&after);
                matches.push((i + 1, l)); // 1-index line numbers
            }
        }
        if !matches.is_empty() {
            if state.is_recursive {
                if state.is_first_match {
                    state.is_first_match = false;
                } else {
                    println!();
                }
                println!("{}{}{}", name_color, path, reset);
            }
            let width = matches[matches.len() - 1].0.to_string().len();
            for (i, line) in matches {
                println!("{}{:>width$}:{} {}", line_color, i, reset, line, width = width);
            }
        }
    }
}
