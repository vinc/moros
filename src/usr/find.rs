use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::regex::Regex;
use crate::sys;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::iter::FromIterator;

struct Options {
    is_first_match: bool,
    is_recursive: bool,
    file: String,
    line: String,
    trim: String,
}

impl Options {
    fn new() -> Self {
        Self {
            is_first_match: true,
            is_recursive: false,
            file: "*".into(),
            line: "".into(),
            trim: "".into(),
        }
    }
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut path = String::new();
    let mut options = Options::new();
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                usage();
                return Ok(());
            }
            "-f" | "--file" => {
                if i + 1 < n {
                    i += 1;
                    options.file = args[i].into();
                } else {
                    error!("Missing file pattern");
                    return Err(ExitCode::UsageError);
                }
            }
            "-l" | "--line" => {
                if i + 1 < n {
                    i += 1;
                    options.line = args[i].into();
                } else {
                    error!("Missing line pattern");
                    return Err(ExitCode::UsageError);
                }
            }
            _ => {
                if path.is_empty() {
                    path = args[i].into();
                } else {
                    error!("Multiple paths not supported");
                    return Err(ExitCode::UsageError);
                }
            }
        }
        i += 1;
    }

    if path.is_empty() {
        path = sys::process::dir();
        options.trim = format!("{}/", path);
    }

    if path.len() > 1 {
        path = path.trim_end_matches('/').into();
    }

    if fs::is_dir(&path) || (fs::is_file(&path) && !options.line.is_empty()) {
        search_files(&path, &mut options);
    } else {
        error!("Invalid path");
        return Err(ExitCode::UsageError);
    }

    Ok(())
}

fn search_files(path: &str, options: &mut Options) {
    if let Ok(mut files) = fs::read_dir(path) {
        files.sort_by_key(|f| f.name());
        options.is_recursive = true;
        for file in files {
            let mut file_path = path.to_string();
            if !file_path.ends_with('/') {
                file_path.push('/');
            }
            file_path.push_str(&file.name());
            if file.is_dir() {
                search_files(&file_path, options);
            } else if is_matching_file(&file_path, &options.file) {
                if options.line == "" {
                    println!("{}", file_path.trim_start_matches(&options.trim));
                } else {
                    print_matching_lines(&file_path, options);
                }
            }

        }
    } else {
        print_matching_lines(path, options);
    }
}

fn is_matching_file(path: &str, pattern: &str) -> bool {
    let file = fs::filename(&path);
    let re = Regex::from_glob(pattern);
    re.is_match(file)
}

fn print_matching_lines(path: &str, options: &mut Options) {
    if !fs::is_file(path) {
        return;
    }

    let file_color = Style::color("yellow");
    let line_color = Style::color("aqua");
    let match_color = Style::color("red");
    let reset = Style::reset();

    let re = Regex::new(&options.line);
    if let Ok(contents) = fs::read_to_string(path) {
        let mut matches = Vec::new();
        for (i, line) in contents.lines().enumerate() {
            let line: Vec<char> = line.chars().collect();
            let mut l = String::new();
            let mut j = 0;
            while let Some((a, b)) = re.find(&String::from_iter(&line[j..])) {
                let m = j + a;
                let n = j + b;
                let b = String::from_iter(&line[j..m]);
                let matched = String::from_iter(&line[m..n]);
                l = format!("{}{}{}{}{}", l, b, match_color, matched, reset);
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
            if options.is_recursive {
                if options.is_first_match {
                    options.is_first_match = false;
                } else {
                    println!();
                }
                println!("{}{}{}", file_color, path, reset);
            }
            let width = matches[matches.len() - 1].0.to_string().len();
            for (i, line) in matches {
                println!(
                    "{}{:>width$}:{} {}",
                    line_color,
                    i,
                    reset,
                    line,
                    width = width
                );
            }
        }
    }
}

fn usage() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} find {}<options> [<path>]{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-f{1}, {0}--file \"<pattern>\"{1}    \
        Find files matching {0}<pattern>{1}",
        csi_option, csi_reset
    );
    println!(
        "  {0}-l{1}, {0}--line \"<pattern>\"{1}    \
        Find lines matching {0}<pattern>{1}",
        csi_option, csi_reset
    );
}

#[test_case]
fn test_find() {
    use crate::{api, usr, sys};
    use crate::usr::shell::exec;

    sys::fs::mount_mem();
    sys::fs::format_mem();
    usr::install::copy_files(false);

    exec("find / => /tmp/find.log").ok();
    assert!(api::fs::read_to_string("/tmp/find.log").unwrap().
        contains("/tmp/alice.txt"));

    exec("find /dev => /tmp/find.log").ok();
    assert!(api::fs::read_to_string("/tmp/find.log").unwrap().
        contains("/dev/random"));

    exec("find /tmp/alice.txt --line Alice => /tmp/find.log").ok();
    assert!(api::fs::read_to_string("/tmp/find.log").unwrap().
        contains("Alice"));

    exec("find nope 2=> /tmp/find.log").ok();
    assert!(api::fs::read_to_string("/tmp/find.log").unwrap().
        contains("Invalid path"));

    exec("find /tmp/alice.txt 2=> /tmp/find.log").ok();
    assert!(api::fs::read_to_string("/tmp/find.log").unwrap().
        contains("Invalid path"));

    exec("find /dev/random --line nope 2=> /tmp/find.log").ok();
    assert!(api::fs::read_to_string("/tmp/find.log").unwrap().
        contains("Invalid path"));

    exec("find /tmp --line list => /tmp/find.log").ok();
    assert!(api::fs::read_to_string("/tmp/find.log").unwrap().
        contains("alice.txt"));

    exec("find /tmp --file \"*.lsp\" --line list => /tmp/find.log").ok();
    assert!(!api::fs::read_to_string("/tmp/find.log").unwrap().
        contains("alice.txt"));

    sys::fs::dismount();
}
