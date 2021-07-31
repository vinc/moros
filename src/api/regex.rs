use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::RangeBounds;

// See "A Regular Expression Matcher" by Rob Pike and Brian Kernighan (2007)

#[derive(Debug)]
pub struct Regex(String);

impl Regex {
    pub fn new(re: &str) -> Self {
        //println!("debug: Regex::new({:?})", re);
        Self(re.to_string())
    }
    pub fn is_match(&self, text: &str) -> bool {
        //println!("debug: {:?}.is_match({:?})", self, text);
        let vec_re: Vec<char> = self.0.chars().collect();
        let vec_text: Vec<char> = text.chars().collect();
        is_match(&vec_re[..], &vec_text[..])
    }
}

fn is_match(re: &[char], text: &[char]) -> bool {
    //println!("debug: is_match({:?}, {:?})", re, text);
    if re.len() == 0 {
        return true;
    }
    if re[0] == '^' {
        return is_match_here(&re[1..], text);
    }
    let mut i = 0;
    let n = text.len();
    loop {
        if is_match_here(re, &text[i..]) {
            return true;
        }
        if i == n {
            return false;
        }
        i += 1;
    }
}

fn is_match_here(re: &[char], text: &[char]) -> bool {
    //println!("debug: is_match_here({:?}, {:?})", re, text);
    if re.len() == 0 {
        return true;
    }
    match re[0] {
        '\\' => return is_match_back(&re[1..], text),
        '$' => return text.len() == 0,
        _ => {},
    }
    if re.len() > 1 {
        match re[1] {
            '*' => return is_match_star(re[0], &re[2..], text),
            '+' => return is_match_plus(re[0], &re[2..], text),
            '?' => return is_match_ques(re[0], &re[2..], text),
            _ => {}
        }
    }
    if text.len() != 0 && (re[0] == '.' || re[0] == text[0]) {
        return is_match_here(&re[1..], &text[1..]);
    }
    false
}

fn is_match_back(re: &[char], text: &[char]) -> bool {
    //println!("debug: is_match_back({:?}, {:?}", re, text);
    if re.len() > 0 && text.len() > 0 {
        match re[0] {
            'D' => if text[0].is_numeric()       { return false },
            'S' => if text[0].is_whitespace()    { return false },
            'W' => if text[0].is_alphanumeric()  { return false },
            'd' => if !text[0].is_numeric()      { return false },
            's' => if !text[0].is_whitespace()   { return false },
            'w' => if !text[0].is_alphanumeric() { return false },
            _   => if text[0] != re[0]           { return false },
        }
        return is_match_here(&re[1..], &text[1..]);
    }
    false
}

fn is_match_star(c: char, re: &[char], text: &[char]) -> bool {
    //println!("debug: is_match_star({:?}, {:?}, {:?}", c, re, text);
    is_match_char(c, re, text, ..)
}

fn is_match_plus(c: char, re: &[char], text: &[char]) -> bool {
    //println!("debug: is_match_plus({:?}, {:?}, {:?}", c, re, text);
    is_match_char(c, re, text, 1..)
}

fn is_match_ques(c: char, re: &[char], text: &[char]) -> bool {
    //println!("debug: is_match_ques({:?}, {:?}, {:?}", c, re, text);
    is_match_char(c, re, text, ..2)
}

fn is_match_char<T: RangeBounds<usize>>(c: char, re: &[char], text: &[char], range: T) -> bool {
    //println!("debug: is_match_char({:?}, {:?}, {:?}", c, re, text);
    let mut i = 0;
    let n = text.len();
    loop {
        if is_match_here(re, &text[i..]) && range.contains(&i) {
            return true;
        }
        if i == n || !(text[i] == c || c == '.') {
            return false;
        }
        i += 1;
    }
}

#[test_case]
fn test_regex() {
    let tests = [
        ("",            "aaa",     true),
        ("",            "",        true),
        ("aaa",         "aaa",     true),
        ("aaa",         "bbb",     false),
        ("a.a",         "aaa",     true),
        ("a.a",         "aba",     true),
        ("a.a",         "abb",     false),

        ("a*",          "aaa",     true),
        ("a*b",         "aab",     true),
        ("a*b*",        "aabb",    true),
        ("a*b*",        "bb",      true),
        ("a.*",         "abb",     true),
        (".*",          "aaa",     true),
        ("a.*",         "a",       true),

        ("a.+",         "ab",      true),
        ("a.+",         "abb",     true),
        ("a.+",         "a",       false),
        ("a.+b",        "ab",      false),
        ("a.+b",        "abb",     true),
        (".+",          "abb",     true),
        (".+",          "b",       true),

        ("a?b",         "abb",     true),
        ("a?b",         "bb",      true),
        ("a?b",         "aabb",    true),

        ("^a.*a$",      "aaa",     true),
        ("^#.*",        "#aaa",    true),
        ("^#.*",        "a#aaa",   false),
        (".*;$",        "aaa;",    true),
        (".*;$",        "aaa;a",   false),
        ("^.*$",        "aaa",     true),

        ("a.b",         "abb",     true),
        ("a.b",         "a.b",     true),
        ("a\\.b",       "abb",     false),
        ("a\\.b",       "a.b",     true),
        ("a\\\\.b",     "abb",     false),
        ("a\\\\.b",     "a.b",     false),
        ("a\\\\.b",     "a\\bb",   true),
        ("a\\\\.b",     "a\\.b",   true),
        ("a\\\\\\.b",   "a\\bb",   false),
        ("a\\\\\\.b",   "a\\.b",   true),
        ("a\\\\\\.b",   "a\\\\bb", false),
        ("a\\\\\\.b",   "a\\\\.b", false),
        ("a\\\\\\\\.b", "a\\bb",   false),
        ("a\\\\\\\\.b", "a\\.b",   false),
        ("a\\\\\\\\.b", "a\\\\bb", true),
        ("a\\\\\\\\.b", "a\\\\.b", true),

        ("a\\wb",       "aéb",     true),
        ("a\\wb",       "awb",     true),
        ("a\\wb",       "abb",     true),
        ("a\\wb",       "a1b",     true),
        ("a\\wb",       "a.b",     false),
        ("a\\Wb",       "aWb",     false),
        ("a\\Wb",       "abb",     false),
        ("a\\Wb",       "a1b",     false),
        ("a\\Wb",       "a.b",     true),
        ("a\\db",       "abb",     false),
        ("a\\db",       "a1b",     true),
        ("a\\Db",       "abb",     true),
        ("a\\Db",       "a1b",     false),
        ("a\\sb",       "abb",     false),
        ("a\\sb",       "a b",     true),
        ("a\\Sb",       "abb",     true),
        ("a\\Sb",       "a b",     false),
    ];
    for (re, text, is_match) in tests {
        assert!(Regex::new(re).is_match(text) == is_match, "Regex::new(\"{}\").is_match(\"{}\") == {}", re, text, is_match);
    }
}
