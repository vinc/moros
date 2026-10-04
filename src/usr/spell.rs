use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;

use alloc::collections::btree_set::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;

const DEFAULT_DICT: &str = "/lib/spell/english.dict";

type Dict = BTreeSet<String>;

fn levenshtein_distance(s: &str, t: &str) -> usize {
    let n = s.chars().count();
    let m = t.chars().count();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }

    let mut d = vec![vec![0; m + 1]; n + 1];

    for i in 0..=n {
        d[i][0] = i;
    }
    for j in 0..=m {
        d[0][j] = j;
    }
    for (i, cs) in s.chars().enumerate() {
        for (j, ct) in t.chars().enumerate() {
            let cost = if cs == ct { 0 } else { 1 };
            d[i + 1][j + 1] = d[i][j + 1].min(d[i + 1][j]).min(d[i][j]) + cost;
        }
    }

    d[n][m]
}

fn find_closest_match(dict: &Dict, word: &str) -> Option<String> {
    let max_prefix = 3;
    let max_distance = 5;
    let mut best_distance = usize::MAX;
    let mut best_candidate = None;

    let n = word.len().min(max_prefix) + 1;
    for i in 1..n {
        let prefix: String = word.chars().take(i).collect();

        for candidate in dict.range(prefix.clone()..) {
            if !candidate.starts_with(&prefix) {
                break;
            }

            let distance = candidate.len().abs_diff(word.len());
            if distance > max_distance || distance >= best_distance {
                continue;
            }

            let distance = levenshtein_distance(word, candidate);
            if distance < best_distance {
                best_distance = distance;
                best_candidate = Some(candidate.clone());

                if distance <= 1 {
                    break;
                }
            }
        }

        if best_distance <= 1 {
            break;
        }
    }

    best_candidate
}

fn spellcheck(dict: &Dict, word: &str) -> bool {
    word.is_empty() || dict.contains(word) || (
        word.find(char::is_uppercase).is_some() &&
        dict.contains(&word.to_lowercase())
    )
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut path = String::new();
    let mut dict = DEFAULT_DICT;
    let mut suggest = false;
    let mut verbose = false;
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
                    dict = args[i];
                } else {
                    error!("Missing dictionary path");
                    return Err(ExitCode::UsageError);
                }
            }
            "-s" | "--suggest" => {
                suggest = true;
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            _ => {
                if args[i].starts_with('-') {
                    error!("Invalid option {:?}", args[i]);
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

    if path.is_empty() {
        help();
        return Err(ExitCode::UsageError);
    }

    let dict: Dict = fs::read_to_string(dict).map(|contents| {
        contents.lines().map(|line| line.trim().into()).collect()
    }).unwrap_or_default();

    if let Ok(buf) = fs::read_to_string(&path) {
        let mut row = 1;
        for line in buf.lines() {
            let mut col = 1;
            let mut word = String::new();
            for c in line.chars() {
                // Recognize "isn't" and "parents'" but not "'quote'"
                if c.is_alphabetic() || (c == '\'' && !word.is_empty()) {
                    word.push(c);
                    col += 1;
                    continue;
                }

                // Transform "parents'" into "parents"
                if word.ends_with('\'') {
                    word.pop();
                    col -= 1;
                }

                if !spellcheck(&dict, &word) {
                    let len = word.chars().count();
                    let col = col - len;
                    error!("Unknown word \"{word}\" at {path}:{row}:{col}");

                    let error = Style::color("red");
                    let reset = Style::reset();
                    if suggest {
                        if let Some(w) = find_closest_match(&dict, &word) {
                            eprintln!("       Did you mean \"{w}\"?");
                        }
                    }
                    if verbose {
                        let mut line = line.to_string();
                        line.insert_str(col - 1 + len, &format!("{}", reset));
                        line.insert_str(col - 1, &format!("{}", error));
                        let space = " ".repeat(col - 1);
                        let arrow = "^".repeat(len);
                        eprintln!("\n{line}\n{space}{error}{arrow}{reset}");
                    }
                }

                word.clear();
                col += 1;
            }
            row += 1;
        }
        Ok(())
    } else {
        error!("Could not read {:?}", path);
        Err(ExitCode::Failure)
    }
}

fn help() {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} spell {}<options> <path>{1}",
        csi_title, csi_reset, csi_option
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-d{1}, {0}--dict <path>{1}    Load dictionary {0}<path>{1}",
        csi_option, csi_reset
    );
    println!(
        "  {0}-s{1}, {0}--suggest{1}        Display suggestion",
        csi_option, csi_reset
    );
    println!(
        "  {0}-v{1}, {0}--verbose{1}        Increase verbosity",
        csi_option, csi_reset
    );
}

#[test_case]
fn test_levenshtein_distance() {
    assert_eq!(levenshtein_distance("kitten", "kitten"), 0);
    assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
}

#[test_case]
fn test_find_closest_match() {
    let dict = vec![
        "aaaaa".to_string(),
        "abcde".to_string(),
        "bbbbb".to_string(),
    ].into_iter().collect();
    assert_eq!(find_closest_match(&dict, "aaaaa"), Some("aaaaa".to_string()));
    assert_eq!(find_closest_match(&dict, "abcda"), Some("abcde".to_string()));
    assert_eq!(find_closest_match(&dict, "bbbba"), Some("bbbbb".to_string()));
}

#[test_case]
fn test_spellcheck() {
    let dict = vec![
        "the".to_string(),
        "quick".to_string(),
        "brown".to_string(),
        "fox".to_string(),
    ].into_iter().collect();
    assert_eq!(spellcheck(&dict, "the"), true);
    assert_eq!(spellcheck(&dict, "The"), true);
    assert_eq!(spellcheck(&dict, "fox"), true);
    assert_eq!(spellcheck(&dict, "dog"), false);
}
