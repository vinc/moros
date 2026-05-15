use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::escaped;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::character::complete::multispace1;
use nom::character::complete::not_line_ending;
use nom::character::complete::one_of;
use nom::character::complete::space0;
use nom::combinator::all_consuming;
use nom::combinator::map;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::separated_pair;
use nom::sequence::terminated;

type ConfigMap = BTreeMap<String, String>;

fn parse_comment(input: &str) -> IResult<&str, &str> {
    preceded(char('#'), not_line_ending).parse(input)
}

fn ignored(input: &str) -> IResult<&str, ()> {
    map(many0(alt((multispace1, parse_comment))), |_| ()).parse(input)
}

fn parse_str(input: &str) -> IResult<&str, &str> {
    delimited(
        char('"'),
        opt(escaped(
            is_not("\\\""),
            '\\',
            one_of("nrt\"\\be")
        )),
        char('"')
    ).map(|res| res.unwrap_or("")).parse(input)
}

fn parse_val(input: &str) -> IResult<&str, &str> {
    alt((parse_str, is_not(" \t\r\n#="))).parse(input)
}

fn parse_eq(input: &str) -> IResult<&str, &str> {
    delimited(space0, tag("="), space0).parse(input)
}

fn parse_key(input: &str) -> IResult<&str, &str> {
    is_not(" \t\r\n#=").parse(input)
}

fn parse_pair(input: &str) -> IResult<&str, (&str, &str)> {
    preceded(
        ignored,
        separated_pair(parse_key, parse_eq, parse_val)
    ).parse(input)
}

fn parse_pairs(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    terminated(many0(parse_pair), ignored).parse(input)
}

pub fn parse(input: &str) -> Option<ConfigMap> {
    let (_, pairs) = all_consuming(parse_pairs).parse(input).ok()?;
    let mut config = ConfigMap::new();
    for (key, val) in pairs {
        config.insert(key.to_string(), val.to_string());
    }
    Some(config)
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
fn test_parse_with_crlf() {
    let input = "key1=value1\r\nkey2=value2\r\n";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
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
