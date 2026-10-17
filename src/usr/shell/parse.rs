
use alloc::vec::Vec;
use crate::api::parse::comment;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::escaped;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::anychar;
use nom::character::complete::char;
use nom::character::complete::one_of;
use nom::character::complete::space0;
use nom::combinator::eof;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::combinator::value;
use nom::multi::many0;
use nom::multi::many1;
use nom::multi::separated_list0;
use nom::sequence::preceded;
use nom::sequence::terminated;

fn blank(input: &str) -> IResult<&str, ()> {
    value((), (space0, opt(comment))).parse(input)
}

fn separator(input: &str) -> IResult<&str, char> {
    preceded(blank, one_of(";\n")).parse(input)
}

fn quoted(input: &str) -> IResult<&str, &str> {
    recognize((
        char('"'),
        opt(escaped(is_not("\\\""), '\\', anychar)),
        char('"'),
    )).parse(input)
}

fn substitution(input: &str) -> IResult<&str, &str> {
    recognize((
        tag("$("),
        many0(alt((quoted, substitution, is_not("\"()")))),
        char(')'),
    )).parse(input)
}

fn command(input: &str) -> IResult<&str, &str> {
    recognize(many1(alt((
        quoted,
        substitution,
        tag("$"),
        is_not("\"$;#\n"),
    )))).parse(input)
}

pub fn parse(input: &str) -> IResult<&str, Vec<&str>> {
    terminated(
        preceded(many0(separator), separated_list0(many1(separator), command)),
        (many0(separator), blank, eof),
    ).parse(input)
}

#[test_case]
fn test_separator() {
    assert_eq!(separator(";"), Ok(("", ';')));
    assert_eq!(separator("; print foo"), Ok((" print foo", ';')));

    assert_eq!(separator("\n"), Ok(("", '\n')));
    assert_eq!(separator("\nprint foo"), Ok(("print foo", '\n')));
    assert_eq!(separator("# comment \nprint foo"), Ok(("print foo", '\n')));
    assert_eq!(separator("# print bar; nope\nprint foo"), Ok(("print foo", '\n')));

    assert!(separator("").is_err());
    assert!(separator("foo").is_err());
    assert!(separator("# comment").is_err());
}

#[test_case]
fn test_quoted() {
    assert_eq!(quoted("\"\""), Ok(("", "\"\"")));
    assert_eq!(quoted("\"foo bar\""), Ok(("", "\"foo bar\"")));
    assert_eq!(quoted("\"foo; bar\""), Ok(("", "\"foo; bar\"")));
    assert_eq!(quoted("\"foo bar\" nope"), Ok((" nope", "\"foo bar\"")));
}

#[test_case]
fn test_substitution() {
    assert_eq!(substitution("$(print foo)"), Ok(("", "$(print foo)")));
    assert_eq!(substitution("$(print foo # bar)"), Ok(("", "$(print foo # bar)")));
    assert_eq!(substitution("$(print \"(foo)\")"), Ok(("", "$(print \"(foo)\")")));
}

#[test_case]
fn test_command() {
    assert_eq!(command("print foo"), Ok(("", "print foo")));
    assert_eq!(command("print foo; bar"), Ok(("; bar", "print foo")));
    assert_eq!(command("print foo # bar"), Ok(("# bar", "print foo ")));
}

#[test_case]
fn test_parse() {
    use alloc::vec;
    assert_eq!(parse("print foo"), Ok(("", vec!["print foo"])));
    assert_eq!(parse("print foo; bar"), Ok(("", vec!["print foo", " bar"])));
}
