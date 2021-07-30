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
    println!("debug: is_match_here('{}', '{}')", re, text);
    if re.len() == 0 {
        return true;
    }
    match re.chars().nth(1) {
        Some('?') => return is_match_ques(re.chars().nth(0).unwrap(), &re[2..], text),
        Some('*') => return is_match_star(re.chars().nth(0).unwrap(), &re[2..], text),
        Some('+') => return is_match_plus(re.chars().nth(0).unwrap(), &re[2..], text),
        _ => {}
    }
    if re.chars().nth(0) == Some('$') && re.len() == 1 {
        return text.len() == 0;
    }
    if text.len() != 0 && (re.chars().nth(0) == Some('.') || re.chars().nth(0) == text.chars().nth(0)) {
        return is_match_here(&re[1..], &text[1..]);
    }
    false
}

fn is_match_ques(c: char, re: &str, text: &str) -> bool {
    //println!("debug: is_match_ques('{}', '{}', '{}')", c, re, text);

    let mut i = 0;
    let n = text.len();
    loop {
        if is_match_here(re, &text[i..]) && i < 2 {
            return true;
        }
        if i == n {
            return false;
        }
        i += 1;
        if !(text.chars().nth(i) == Some(c) || c == '.') {
            return false;
        }
    }
}

fn is_match_star(c: char, re: &str, text: &str) -> bool {
    //println!("debug: is_match_star('{}', '{}', '{}')", c, re, text);

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
        if !(text.chars().nth(i) == Some(c) || c == '.') {
            return false;
        }
    }
}

fn is_match_plus(c: char, re: &str, text: &str) -> bool {
    //println!("debug: is_match_plus('{}', '{}', '{}')", c, re, text);

    let mut i = 0;
    let n = text.len();
    loop {
        if is_match_here(re, &text[i..]) && i > 0 {
            return true;
        }
        if i == n {
            return false;
        }
        i += 1;
        if !(text.chars().nth(i) == Some(c) || c == '.') {
            return false;
        }
    }
}

#[test_case]
fn test_regex() {
    let tests = [
        ("aaa",    "aaa",   true),
        ("aaa",    "bbb",   false),
        ("a.a",    "aaa",   true),
        ("a.a",    "aba",   true),
        ("a.a",    "abb",   false),

        ("a?b",    "abb",   true),
        ("a?b",    "bb",    true),
        ("a?b",    "aabb",  true),

        ("a*",     "aaa",   true),
        ("a*b",    "aab",   true),
        ("a*b*",   "aabb",  true),
        ("a*b*",   "bb",    true),
        ("a.*",    "abb",   true),
        (".*",     "aaa",   true),
        ("a.*",    "a",     true),

        ("a.+",    "ab",    true),
        ("a.+",    "abb",   true),
        ("a.+",    "a",     false),
        ("a.+b",   "ab",    false),
        ("a.+b",   "abb",   true),
        (".+",     "abb",   true),
        (".+",     "b",     true),

        ("^a.*a$", "aaa",   true),
        ("^#.*",   "#aaa",  true),
        ("^#.*",   "a#aaa", false),
        (".*;$",   "aaa;",  true),
        (".*;$",   "aaa;a", false),
        ("^.*$",   "aaa",   true),
        ("^.*$",   "aaa",   true),
    ];
    for (re, text, is_match) in tests {
        assert!(Regex::new(re).is_match(text) == is_match, "Regex::new(\"{}\").is_match(\"{}\") == {}", re, text, is_match);
    }
}
