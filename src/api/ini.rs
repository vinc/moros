use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use chumsky::prelude::*;

type ConfigMap = BTreeMap<String, String>;
type ParseResult<'a> = ConfigMap;
type ParseError<'a> = extra::Err<Simple<'a, char>>;

fn parser<'a>() -> impl Parser<'a, &'a str, ParseResult<'a>, ParseError<'a>> {
    let whitespace = one_of(" \t").repeated();

    let key = text::ident();

    let val = none_of("\n\r").
        repeated().
        collect::<String>().
        map(|s| s.trim().to_string());

    let pair = key.
        padded_by(whitespace.clone()).
        then_ignore(just('=')).
        padded_by(whitespace.clone()).
        then(val).
        map(|(k, v): (&str, String)| (k.to_string(), v));

    pair.padded_by(text::newline().repeated()).
        repeated().
        collect::<Vec<_>>().
        map(|pairs| pairs.into_iter().collect())
}

pub fn parse(input: &str) -> Option<ConfigMap> {
    parser().parse(input).into_result().ok()
}

#[test_case]
fn test_parse() {
    let input = "key1=value1\nkey2=value2";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_whitespace() {
    let input = "key1 = value1\nkey2 = value2";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_empty_lines() {
    let input = "key1=value1\n\nkey2=value2\n";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_spaces_in_values() {
    let input = "key1=  value with spaces  \nkey2=another value";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value with spaces".to_string()),
        ("key2".to_string(), "another value".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_crlf() {
    let input = "key1=value1\r\nkey2=value2\r\n";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_empty_value() {
    let input = "key1=\nkey2=value2";
    let expected = BTreeMap::from([
        ("key1".to_string(), "".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_special_chars_in_value() {
    let input = "path=/usr/bin/test\nurl=https://example.com:8080";
    let expected = BTreeMap::from([
        ("path".to_string(), "/usr/bin/test".to_string()),
        ("url".to_string(), "https://example.com:8080".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_special_chars_in_key() {
    let input = "path=/usr/bin/test\nurl=https://example.com:8080";
    let expected = BTreeMap::from([
        ("path".to_string(), "/usr/bin/test".to_string()),
        ("url".to_string(), "https://example.com:8080".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}
