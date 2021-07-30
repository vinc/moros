use alloc::string::{String, ToString};

// See "A Regular Expression Matcher" by Rob Pike and Brian Kernighan (2007)

pub struct Regex(String);

impl Regex {
    pub fn new(re: &str) -> Self {
        //println!("debug: new('{}')", re);
        Self(re.to_string())
    }
    pub fn is_match(&self, text: &str) -> bool {
        is_match(&self.0, text)
    }
}

fn is_match(re: &str, text: &str) -> bool {
    //println!("debug: is_match('{}', '{}')", re, text);
    if re.chars().nth(0) == Some('^') {
        return is_match_here(&re[1..], text);
    }
    let mut i = 0;
    let n = text.len();
    while i < n {
        if is_match_here(re, &text[i..]) {
            return true;
        }
        i += 1;
    }
    false
}

fn is_match_here(re: &str, text: &str) -> bool {
    //println!("debug: is_match_here('{}', '{}')", re, text);
    if re.len() == 0 {
        return true;
    }
    if re.chars().nth(1) == Some('*') {
        return is_match_star(re.chars().nth(0).unwrap(), &re[2..], text);
    }
    if re.chars().nth(1) == Some('+') {
        return is_match_plus(re.chars().nth(0).unwrap(), &re[2..], text);
    }
    if re.chars().nth(0) == Some('$') && re.len() == 1 {
        return text.len() == 0;
    }
    if text.len() != 0 && (re.chars().nth(0) == Some('.') || re.chars().nth(0) == text.chars().nth(0)) {
        return is_match_here(&re[1..], &text[1..]);
    }
    false
}

fn is_match_star(c: char, re: &str, text: &str) -> bool {
    //println!("debug: is_match_star('{}', '{}', '{}')", c, re, text);
    let mut i = 0;
    let n = text.len();
    while i <= n && (text.chars().nth(i) == Some(c) || c == '.') {
        if is_match_here(re, &text[i..]) {
            return true;
        }
        i += 1;
    }
    false
}

fn is_match_plus(c: char, re: &str, text: &str) -> bool {
    println!("debug: is_match_star('{}', '{}', '{}')", c, re, text);
    let mut i = 0;
    let n = text.len();
    while i <= n && (text.chars().nth(i) == Some(c) || c == '.') {
        if is_match_here(re, &text[i..]) && i > 0 {
            return true;
        }
        i += 1;
    }
    false
}

#[test_case]
fn test_regex() {
    assert_eq!(Regex::new("aaa").is_match("aaa"), true);
    assert_eq!(Regex::new("aaa").is_match("bbb"), false);
    assert_eq!(Regex::new("a.a").is_match("aaa"), true);
    assert_eq!(Regex::new("a.a").is_match("aba"), true);
    assert_eq!(Regex::new("a.a").is_match("abb"), false);

    assert_eq!(Regex::new("a*").is_match("aaa"), true);
    //assert_eq!(Regex::new("a*b").is_match("aab"), true); // FIXME
    //assert_eq!(Regex::new("a*b*").is_match("aabb"), true); // FIXME
    assert_eq!(Regex::new("a.*").is_match("abb"), true);
    assert_eq!(Regex::new(".*").is_match("aaa"), true);
    assert_eq!(Regex::new("a.*").is_match("a"), true);

    assert_eq!(Regex::new("a.+").is_match("ab"), true);
    assert_eq!(Regex::new("a.+").is_match("abb"), true);
    assert_eq!(Regex::new("a.+").is_match("a"), false);
    assert_eq!(Regex::new("a.+b").is_match("ab"), false);
    assert_eq!(Regex::new("a.+b").is_match("abb"), true);
    assert_eq!(Regex::new(".+").is_match("abb"), true);
    assert_eq!(Regex::new(".+").is_match("b"), true);

    assert_eq!(Regex::new("^a.*a$").is_match("aaa"), true);
    assert_eq!(Regex::new("^#.*").is_match("#aaa"), true);
    assert_eq!(Regex::new("^#.*").is_match("a#aaa"), false);
    assert_eq!(Regex::new(".*;$").is_match("aaa;"), true);
    assert_eq!(Regex::new(".*;$").is_match("aaa;a"), false);
    assert_eq!(Regex::new("^.*$").is_match("aaa"), true);
}
