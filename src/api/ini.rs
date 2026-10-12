use crate::api::parse::{ignored, hexadecimal, decimal, binary, float};

use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::escaped_transform;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::character::complete::space0;
use nom::combinator::all_consuming;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::combinator::value;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::separated_pair;
use nom::sequence::terminated;

type ConfigMap = BTreeMap<String, String>;

fn parse_str(input: &str) -> IResult<&str, String> {
    let escaped = map(
        opt(escaped_transform(
            is_not("\\\""),
            '\\',
            alt((
                value("\\", tag("\\")),
                value("\"", tag("\"")),
                value("\n", tag("n")),
                value("\r", tag("r")),
                value("\t", tag("t")),
                value("\x08", tag("b")),
                value("\x1B", tag("e"))
            ))
        )),
        |inner| inner.unwrap_or("".to_string())
    );
    delimited(char('"'), escaped, char('"')).parse(input)
}

fn parse_val(input: &str) -> IResult<&str, String> {
    alt((
        parse_str,
        recognize((
            opt(alt((char('+'), char('-')))),
            alt((float, hexadecimal, binary, decimal)),
        )).map(|s: &str| s.to_string()),
    )).parse(input)
}

fn parse_key(input: &str) -> IResult<&str, &str> {
    is_not(" \t\r\n#=").parse(input)
}

fn parse_eq(input: &str) -> IResult<&str, &str> {
    delimited(space0, tag("="), space0).parse(input)
}

fn parse_pair(input: &str) -> IResult<&str, (&str, String)> {
    preceded(
        ignored,
        separated_pair(parse_key, parse_eq, parse_val)
    ).parse(input)
}

fn parse_pairs(input: &str) -> IResult<&str, Vec<(&str, String)>> {
    terminated(many0(parse_pair), ignored).parse(input)
}

pub fn parse(input: &str) -> Option<ConfigMap> {
    let (_, pairs) = all_consuming(parse_pairs).parse(input).ok()?;
    let mut config = ConfigMap::new();
    for (key, val) in pairs {
        config.insert(key.to_string(), val);
    }
    Some(config)
}

#[test_case]
fn test_parse() {
    let input = "key1=\"foo\"\nkey2=42";
    let expected = BTreeMap::from([
        ("key1".to_string(), "foo".to_string()),
        ("key2".to_string(), "42".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_whitespace() {
    let input = "key1 = \"foo\"\nkey2 = 42";
    let expected = BTreeMap::from([
        ("key1".to_string(), "foo".to_string()),
        ("key2".to_string(), "42".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_empty_lines() {
    let input = "key1=\"foo\"\n\nkey2=42\n";
    let expected = BTreeMap::from([
        ("key1".to_string(), "foo".to_string()),
        ("key2".to_string(), "42".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_crlf() {
    let input = "key1=\"foo\"\r\nkey2=42\r\n";
    let expected = BTreeMap::from([
        ("key1".to_string(), "foo".to_string()),
        ("key2".to_string(), "42".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_special_chars() {
    let input = "path=\"/usr/bin/test\"\nurl=\"https://example.com:8080\"";
    let expected = BTreeMap::from([
        ("path".to_string(), "/usr/bin/test".to_string()),
        ("url".to_string(), "https://example.com:8080".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}

#[test_case]
fn test_parse_with_comments() {
    let input = "# comment\nkey1 = \"foo\" # comment\nkey2 = 42";
    let expected = BTreeMap::from([
        ("key1".to_string(), "foo".to_string()),
        ("key2".to_string(), "42".to_string()),
    ]);

    assert_eq!(parse(input), Some(expected));
}
