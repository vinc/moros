use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;

use core::cmp;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;

#[derive(Debug, PartialEq)]
enum Diff {
    Equal(String),
    Insert(String),
    Delete(String),
}

fn diff(old: &str, new: &str) -> Vec<Diff> {
    let old: Vec<&str> = old.lines().collect();
    let new: Vec<&str> = new.lines().collect();
    backtrack(&old, &new, &lcs(&old, &new))
}

fn lcs(old: &[&str], new: &[&str]) -> Vec<Vec<usize>> {
    let n = old.len();
    let m = new.len();
    let mut lcs = vec![vec![0; m + 1]; n + 1];

    for i in 0..n {
        for j in 0..m {
            if old[i] == new[j] {
                lcs[i + 1][j + 1] = lcs[i][j] + 1;
            } else {
                lcs[i + 1][j + 1] = cmp::max(lcs[i][j + 1], lcs[i + 1][j]);
            }
        }
    }

    lcs
}

fn backtrack(old: &[&str], new: &[&str], lcs: &[Vec<usize>]) -> Vec<Diff> {
    let mut diff = Vec::new();
    let mut i = old.len();
    let mut j = new.len();

    while i > 0 && j > 0 {
        if old[i - 1] == new[j - 1] {
            diff.push(Diff::Equal(old[i - 1].into()));
            i -= 1;
            j -= 1;
        } else if lcs[i][j - 1] > lcs[i - 1][j] {
            diff.push(Diff::Insert(new[j - 1].into()));
            j -= 1;
        } else {
            diff.push(Diff::Delete(old[i - 1].into()));
            i -= 1;
        }
    }

    while i > 0 {
        diff.push(Diff::Delete(old[i - 1].into()));
        i -= 1;
    }

    while j > 0 {
        diff.push(Diff::Insert(new[j - 1].into()));
        j -= 1;
    }

    diff.reverse();
    diff
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let csi_insert = Style::color("green");
    let csi_delete = Style::color("red");
    let csi_reset = Style::reset();
    if let Ok(old) = fs::read_to_string(args[1]) {
        if let Ok(new) = fs::read_to_string(args[2]) {
            for op in diff(&old, &new) {
                match op {
                    Diff::Equal(line) => {
                        println!(" {}", line);
                    }
                    Diff::Delete(line) => {
                        println!("{}-{}{}", csi_delete, line, csi_reset);
                    }
                    Diff::Insert(line) => {
                        println!("{}+{}{}", csi_insert, line, csi_reset);
                    }
                }
            }
        }
    }
    Ok(())
}

#[test_case]
fn test_diff() {
    assert_eq!(
        diff("a", "a"),
        vec![
            Diff::Equal("a".into())
        ]
    );
    assert_eq!(
        diff("a", "b"),
        vec![
            Diff::Insert("b".into()),
            Diff::Delete("a".into())
        ]
    );
    assert_eq!(
        diff("a\nb\nc\n", "a\nc\n"),
        vec![
            Diff::Equal("a".into()),
            Diff::Delete("b".into()),
            Diff::Equal("c".into())
        ]
    );
}
