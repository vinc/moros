use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use chumsky::prelude::*;

// Parser that takes a string and returns a BTreeMap<String, String>
fn parser<'a>() -> impl Parser<'a, &'a str, BTreeMap<String, String>, extra::Err<Simple<'a, char>>> {
    let whitespace = one_of(" \t").repeated();

    // Parse key - alphanumeric and underscores
    let key = text::ident();

    // Parse value - anything until newline, trimmed
    let value = none_of("\n\r")
        .repeated()
        .collect::<String>()
        .map(|s| s.trim().to_string());

    // Parse key=value pair
    let pair = key
        .padded_by(whitespace.clone())
        .then_ignore(just('='))
        .padded_by(whitespace.clone())
        .then(value)
        .map(|(k, v): (&str, String)| (k.to_string(), v));

    // Parse multiple pairs separated by newlines (including empty lines)
    pair.padded_by(text::newline().repeated())
        .repeated()
        .collect::<Vec<_>>()
        .map(|pairs| pairs.into_iter().collect())
}

/// Parse an INI-formatted string into a BTreeMap of key-value pairs.
///
/// This parser supports:
/// - Keys consisting of alphanumeric characters and underscores
/// - Values containing any characters except newlines
/// - Optional whitespace around the `=` separator
/// - Both LF (`\n`) and CRLF (`\r\n`) line endings
/// - Empty lines between key-value pairs
/// - Trailing whitespace in values (which is trimmed)
///
/// # Examples
///
/// ```
/// let input = "key1=value1\nkey2=value2";
/// let result = parse(input).unwrap();
/// assert_eq!(result.get("key1"), Some(&"value1".to_string()));
/// ```
pub fn parse(input: &str) -> Result<BTreeMap<String, String>, Vec<Simple<char>>> {
    parser().parse(input).into_result()
}

#[test_case]
fn test_parse() {
    let input = "key1=value1\nkey2=value2";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Ok(expected));
}

#[test_case]
fn test_parse_with_whitespace() {
    let input = "key1 = value1\nkey2 = value2";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Ok(expected));
}

#[test_case]
fn test_parse_with_empty_lines() {
    let input = "key1=value1\n\nkey2=value2\n";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Ok(expected));
}

#[test_case]
fn test_parse_with_spaces_in_values() {
    let input = "key1=  value with spaces  \nkey2=another value";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value with spaces".to_string()),
        ("key2".to_string(), "another value".to_string()),
    ]);

    assert_eq!(parse(input), Ok(expected));
}

#[test_case]
fn test_parse_with_crlf() {
    let input = "key1=value1\r\nkey2=value2\r\n";
    let expected = BTreeMap::from([
        ("key1".to_string(), "value1".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Ok(expected));
}

#[test_case]
fn test_parse_with_empty_value() {
    let input = "key1=\nkey2=value2";
    let expected = BTreeMap::from([
        ("key1".to_string(), "".to_string()),
        ("key2".to_string(), "value2".to_string()),
    ]);

    assert_eq!(parse(input), Ok(expected));
}

#[test_case]
fn test_parse_with_special_chars_in_value() {
    let input = "path=/usr/bin/test\nurl=https://example.com:8080";
    let expected = BTreeMap::from([
        ("path".to_string(), "/usr/bin/test".to_string()),
        ("url".to_string(), "https://example.com:8080".to_string()),
    ]);

    assert_eq!(parse(input), Ok(expected));
}

#[test_case]
fn test_parse_with_special_chars_in_key() {
    let input = "path=/usr/bin/test\nurl=https://example.com:8080";
    let expected = BTreeMap::from([
        ("path".to_string(), "/usr/bin/test".to_string()),
        ("url".to_string(), "https://example.com:8080".to_string()),
    ]);

    assert_eq!(parse(input), Ok(expected));
}
