use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use chumsky::prelude::*;

const DEFAULT_DICT: &str = "/lib/spell/english.dict";

fn is_word_char(c: &char) -> bool {
    c.is_alphabetic() || *c == '\''
}

fn is_not_word_char(c: &char) -> bool {
    !is_word_char(c)
}

fn parser<'a>(dict: &'a Vec<String>) -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> {
    let non_word = any().
        filter(is_not_word_char).
        repeated();

    let word = any().
        filter(is_word_char).
        repeated().
        at_least(1).
        collect::<String>();

    let valid_word = word.clone().validate(move |word: String, e, emitter| {
        if !dict.contains(&word) && !dict.contains(&word.to_lowercase()) {
            let reason = format!("Unknown word \"{}\"", word);
            emitter.emit(Rich::custom(e.span(), reason));
        }
        word
    });

    valid_word.padded_by(non_word).repeated()
}

fn pos(buf: &str, i: usize) -> (usize, usize) {
    let mut col = 1;
    let mut row = 1;
    let mut j = 0;
    for line in buf.lines() {
        let n = line.len();
        if i < j + n {
            col = i - j + 1;
            break;
        }
        j += n + 1;
        row += 1;
    }
    (row, col)
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let len_a = a.chars().count();
    let len_b = b.chars().count();
    let mut d = vec![vec![0; len_b + 1]; len_a + 1];

    for i in 0..=len_a {
        d[i][0] = i;
    }
    for j in 0..=len_b {
        d[0][j] = j;
    }
    for (i, ca) in a.chars().enumerate() {
        for (j, cb) in b.chars().enumerate() {
            let n = if ca == cb { 0 } else { 1 };
            d[i + 1][j + 1] = d[i][j + 1].min(d[i + 1][j]).min(d[i][j]) + n;
        }
    }

    d[len_a][len_b]
}

fn find_closest_match(dict: &Vec<String>, word: &str) -> Option<String> {
    dict.iter().min_by_key(|&w| levenshtein_distance(word, w)).cloned()
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let mut verbose = false;
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
            "-v" | "--verbose" => {
                verbose = true;
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

    if path.is_empty() {
        help();
        return Err(ExitCode::UsageError);
    }

    let dict: Vec<String> = fs::read_to_string(&dict).map(|contents| {
        contents.lines().map(|line| line.trim().into()).collect()
    }).unwrap_or_default();

    if let Ok(buf) = fs::read_to_string(&path) {
        match parser(&dict).parse(&buf).into_result() {
            Ok(()) => {},
            Err(errs) => errs.into_iter().for_each(|e| {
                let (row, col) = pos(&buf, e.span().start);
                let reason = e.reason().to_string();
                error!("{reason} at {path}:{row}:{col}");

                if verbose {
                    let error = Style::color("red");
                    let reset = Style::reset();

                    let word = &buf[e.span().start..e.span().end];
                    if let Some(suggestion) = find_closest_match(&dict, word) {
                        eprintln!("{error}-----> {reset}Did you mean \"{suggestion}\"?");
                    }

                    let len = e.span().end - e.span().start;
                    let mut line = buf.lines().skip(row - 1).next().unwrap().to_string();
                    line.insert_str(col + len - 1, &format!("{}", reset));
                    line.insert_str(col - 1, &format!("{}", error));
                    let space = " ".repeat(col - 1);
                    let arrow = "^".repeat(e.span().end - e.span().start);
                    eprintln!("\n{line}\n{space}{error}{arrow}{reset}");
                }
            })
        };
        Ok(())
    } else {
        error!("Could not read '{}'", path);
        Err(ExitCode::Failure)
    }
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
        "  {0}-d{1}, {0}--dict \"<path>\"{1}    Load dictionary {0}<path>{1}",
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
    ];
    assert_eq!(find_closest_match(&dict, "aaaaa"), Some("aaaaa".to_string()));
    assert_eq!(find_closest_match(&dict, "abcda"), Some("abcde".to_string()));
    assert_eq!(find_closest_match(&dict, "bbbba"), Some("bbbbb".to_string()));
}
