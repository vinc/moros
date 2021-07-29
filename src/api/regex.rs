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

#[test_case]
fn test_regex() {
    assert!(Regex::new("aaa").is_match("aaa"));
    assert!(!Regex::new("aaa").is_match("bbb"));
    assert!(Regex::new("a.a").is_match("aaa"));
    assert!(Regex::new("a.a").is_match("aba"));
    assert!(!Regex::new("a.a").is_match("abb"));
    assert!(Regex::new("a.*").is_match("abb"));
    assert!(Regex::new(".*").is_match("aaa"));
    assert!(Regex::new("^a.*a$").is_match("aaa"));
    assert!(Regex::new("^#.*").is_match("#aaa"));
    assert!(!Regex::new("^#.*").is_match("a#aaa"));
    assert!(Regex::new(".*;$").is_match("aaa;"));
    assert!(!Regex::new(".*;$").is_match("aaa;a"));
    assert!(Regex::new("^.*$").is_match("aaa"));
}
