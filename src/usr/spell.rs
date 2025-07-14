use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

const DEFAULT_DICT: &str = "/lib/spell/english.dict";

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut path = String::new();
    let mut dict = DEFAULT_DICT;
    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return Ok(());
            }
            "-d" | "--dict" => {
                if i + 1 < n {
                    i += 1;
                    dict = args[i].into();
                } else {
                    error!("Missing dictionary path");
                    return Err(ExitCode::UsageError);
                }
            }
            _ => {
                if args[i].starts_with('-') {
                    error!("Invalid option '{}'", args[i]);
                    return Err(ExitCode::UsageError);
                } else if path.is_empty() {
                    path = args[i].into();
                } else {
                    error!("Multiple paths not supported");
                    return Err(ExitCode::UsageError);
                }
            }
        }
        i += 1;
    }

    if args.len() != 2 {
        help();
        return Err(ExitCode::UsageError);
    }

    let dict: Vec<String> = fs::read_to_string(&dict).map(|contents| {
        contents.lines().map(|line| line.trim().into()).collect()
    }).unwrap_or_default();

    /*
    let dict: Vec<String> = fs::read_to_string("/lib/spell/english.dict").map(|contents| {
        contents.lines().map(|line| line.trim().into()).collect()
    }).unwrap_or_default();

    let dict2: Vec<String> = fs::read_to_string("/lib/spell/english-ext.dict").map(|contents| {
        contents.lines().map(|line| line.trim().into()).collect()
    }).unwrap_or_default();

    let dict = [dict, dict2].concat();
    */

    if let Ok(contents) = fs::read_to_string(&path) {
        for (row, line) in contents.lines().enumerate() {
            let mut j = 0;
            for word in line.split(" ") {
                let col = j;
                let len = word.len();
                j += len + 1;
                let word = word.trim().trim_matches(|c: char| {
                    c.is_ascii_punctuation()
                });
                if word.is_empty() {
                    continue;
                }
                if !word.chars().all(|c| c.is_alphabetic() || c == '\'') {
                    continue;
                }
                if dict.contains(&word.into()) {
                    continue;
                }
                if dict.contains(&word.to_lowercase()) {
                    continue;
                }

                let mut line: String = line.into();
                let error = Style::color("red");
                let reset = Style::reset();
                let space = " ".repeat(col);
                let arrow = "^".repeat(len);
                line.insert_str(col + len, &format!("{}", reset));
                line.insert_str(col, &format!("{}", error));
                error!("Unknown word at {path}:{row}:{col}");
                eprintln!("\n{line}\n{space}{error}{arrow}{reset}\n");
            }
        }
    }

    Ok(())
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} spell {}<options> [<path>]{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-d{1}, {0}--dict \"<path>\"{1}    \
        Load dictionary {0}<path>{1}",
        csi_option, csi_reset
    );
}
