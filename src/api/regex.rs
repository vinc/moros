use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::convert::From;
use core::ops::RangeBounds;

// See "A Regular Expression Matcher" by Rob Pike and Brian Kernighan (2007)

#[derive(Debug)]
enum MetaChar {
    Any,
    Numeric,
    Whitespace,
    Alphanumeric,
    NonNumeric,
    NonWhitespace,
    NonAlphanumeric,
    Literal(char),
}

impl From<char> for MetaChar {
    fn from(c: char) -> Self {
        match c {
            '.' => MetaChar::Any,
            _   => MetaChar::Literal(c),
        }
    }
}

trait MetaCharExt {
    fn from_escaped(c: char) -> Self;
    fn contains(&self, c: char) -> bool;
}

impl MetaCharExt for MetaChar {
    fn from_escaped(c: char) -> Self {
        match c {
            'd' => MetaChar::Numeric,
            's' => MetaChar::Whitespace,
            'w' => MetaChar::Alphanumeric,
            'D' => MetaChar::NonNumeric,
            'S' => MetaChar::NonWhitespace,
            'W' => MetaChar::NonAlphanumeric,
            _   => MetaChar::Literal(c),
        }
    }
    fn contains(&self, c: char) -> bool {
        match self {
            MetaChar::Any => true,
            MetaChar::Numeric => c.is_numeric(),
            MetaChar::Whitespace => c.is_whitespace(),
            MetaChar::Alphanumeric => c.is_alphanumeric(),
            MetaChar::NonNumeric => !c.is_numeric(),
            MetaChar::NonWhitespace => !c.is_whitespace(),
            MetaChar::NonAlphanumeric => !c.is_alphanumeric(),
            MetaChar::Literal(lc) => c == *lc,
        }
    }
}

#[derive(Debug)]
pub struct Regex(String);

impl Regex {
    pub fn new(re: &str) -> Self {
        Self(re.to_string())
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.find(text).is_some()
    }

    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        let text: Vec<char> = text.chars().collect(); // UTF-32
        let re: Vec<char> = self.0.chars().collect(); // UTF-32
        let mut start = 0;
        let mut end = 0;
        if is_match(&re[..], &text[..], &mut start, &mut end) {
            Some((start, end))
        } else {
            None
        }
    }
}

fn is_match(re: &[char], text: &[char], start: &mut usize, end: &mut usize) -> bool {
    if re.is_empty() {
        return true;
    }
    if re[0] == '^' {
        *end = 1;
        return is_match_here(&re[1..], text, end);
    }
    let mut i = 0;
    let n = text.len();
    loop {
        *start = i;
        *end = i;
        if is_match_here(re, &text[i..], end) {
            return true;
        }
        if i == n {
            return false;
        }
        i += 1;
    }
}

fn is_match_here(re: &[char], text: &[char], end: &mut usize) -> bool {
    if re.is_empty() {
        return true;
    }
    if re[0] == '$' {
        return text.is_empty();
    }
    let (mc, i) = if re.len() > 1 && re[0] == '\\' {
        (MetaChar::from_escaped(re[1]), 1)
    } else {
        (MetaChar::from(re[0]), 0)
    };
    if re.len() > i + 1 {
        let lazy = re.len() > i + 2 && re[i + 2] == '?';
        let j = if lazy { i + 3 } else { i + 2 };

        match re[i + 1] {
            '*' => return is_match_star(lazy, mc, &re[j..], text, end),
            '+' => return is_match_plus(lazy, mc, &re[j..], text, end),
            '?' => return is_match_ques(lazy, mc, &re[j..], text, end),
            _ => {}
        }
    }
    if !text.is_empty() && mc.contains(text[0]) {
        *end += 1;
        let j = i + 1;
        return is_match_here(&re[j..], &text[1..], end);
    }
    false
}

fn is_match_star(lazy: bool, mc: MetaChar, re: &[char], text: &[char], end: &mut usize) -> bool {
    is_match_char(lazy, mc, re, text, .., end)
}

fn is_match_plus(lazy: bool, mc: MetaChar, re: &[char], text: &[char], end: &mut usize) -> bool {
    is_match_char(lazy, mc, re, text, 1.., end)
}

fn is_match_ques(lazy: bool, mc: MetaChar, re: &[char], text: &[char], end: &mut usize) -> bool {
    is_match_char(lazy, mc, re, text, ..2, end)
}

fn is_match_char<T: RangeBounds<usize>>(lazy: bool, mc: MetaChar, re: &[char], text: &[char], range: T, end: &mut usize) -> bool {
    let mut i = 0;
    let n = text.len();

    if !lazy {
        loop {
            if i == n || !(mc.contains(text[i])) {
                break;
            }
            i += 1;
        }
    }

    loop {
        if is_match_here(re, &text[i..], end) && range.contains(&i) {
            *end += i;
            return true;
        }
        if lazy {
            if i == n || !(mc.contains(text[i])) {
                return false;
            }
            i += 1;
        } else {
            if i == 0 {
                return false;
            }
            i -= 1;
        }
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

        ("a\\.*d",      "a..d",    true),
        ("a\\.*d",      "a.cd",    false),
        ("a\\w*d",      "abcd",    true),
    ];
    for (re, text, is_match) in tests {
        assert!(Regex::new(re).is_match(text) == is_match, "Regex::new(\"{}\").is_match(\"{}\") == {}", re, text, is_match);
    }

    assert_eq!(Regex::new(".*").find("abcd"), Some((0, 4)));
    assert_eq!(Regex::new("b.*c").find("aaabbbcccddd"), Some((3, 9)));
    assert_eq!(Regex::new("b.*?c").find("aaabbbcccddd"), Some((3, 7)));
    assert_eq!(Regex::new("a\\w*d").find("abcdabcd"), Some((0, 8)));
    assert_eq!(Regex::new("a\\w*?d").find("abcdabcd"), Some((0, 4)));
    assert_eq!(Regex::new("\\$\\w+").find("test $test test"), Some((5, 10)));
}
