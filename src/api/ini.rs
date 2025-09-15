use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use chumsky::prelude::*;

type ConfigMap = BTreeMap<String, String>;
type ParseResult<'a> = ConfigMap;
type ParseError<'a> = extra::Err<Simple<'a, char>>;

fn parser<'a>() -> impl Parser<'a, &'a str, ParseResult<'a>, ParseError<'a>> {
    let whitespace = one_of(" \t").repeated();
    let newline = one_of("\n\r").repeated().at_least(1);
    let key = text::ident();
    let comment = just('#').then(none_of("\n\r").repeated()).ignored();

    let quoted_val = none_of("\"").
        repeated().
        collect::<String>().
        delimited_by(just('"'), just('"'));

    let unquoted_val = none_of("\n\r#").
        repeated().
        collect::<String>();

    let val = quoted_val.or(unquoted_val).map(|s| s.trim().to_string());

    let pair = key.
        then_ignore(whitespace).
        then_ignore(just('=')).
        then_ignore(whitespace).
        then(val).
        then_ignore(whitespace).
        then_ignore(comment.or_not()).
        map(|(k, v): (&str, String)| (k.to_string(), v));

    let line = pair.
        map(Some).
        or(comment.to(None)).
        or(whitespace.to(None));

    line.separated_by(newline).
        allow_trailing().
        collect::<Vec<_>>().
        map(|items| items.into_iter().flatten().collect())
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
fn test_parse_with_special_chars() {
    let input = "path=/usr/bin/test\nurl=https://example.com:8080";
    let expected = BTreeMap::from([
        ("path".to_string(), "/usr/bin/test".to_string()),
        ("url".to_string(), "https://example.com:8080".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_quotes() {
    let input = "key1 = \"value1\"\nkey2 = \"value2\"";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_comments() {
    let input = "# comment\nkey1 = value1 # comment\nkey2 = value2";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}
