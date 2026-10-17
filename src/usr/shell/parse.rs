use crate::api::parse::comment;

use alloc::vec::Vec;
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
use nom::character::complete::space1;
use nom::combinator::eof;
use nom::combinator::map;
use nom::combinator::not;
use nom::combinator::opt;
use nom::combinator::peek;
use nom::combinator::recognize;
use nom::combinator::value;
use nom::error::Error;
use nom::multi::many0;
use nom::multi::many1;
use nom::multi::separated_list0;
use nom::sequence::preceded;
use nom::sequence::terminated;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    And,
    Or,
}

#[derive(Debug, PartialEq)]
pub struct Chain<'a> {
    pub head: &'a str,
    pub tail: Vec<(Op, &'a str)>,
}

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

pub fn parse(input: &str) -> IResult<&str, Vec<Chain<'_>>> {
    terminated(
        preceded(many0(separator), separated_list0(many1(separator), chain)),
        (many0(separator), blank, eof),
    ).parse(input)
}

fn keyword<'a>(
    kw: &'static str
) -> impl Parser<&'a str, Output = &'a str, Error = Error<&'a str>> {
    terminated(tag(kw), peek(alt((space1, eof, recognize(one_of(";\n#"))))))
}

fn op(input: &str) -> IResult<&str, Op> {
    alt(
        (value(Op::And, keyword("and")), value(Op::Or, keyword("or")))
    ).parse(input)
}

fn text(input: &str) -> IResult<&str, &str> {
    is_not(" \t\"$;#\n").parse(input)
}

fn part(input: &str) -> IResult<&str, &str> {
    alt((quoted, substitution, tag("$"), text)).parse(input)
}

fn word(input: &str) -> IResult<&str, &str> {
    preceded(not(op), recognize(many1(part))).parse(input)
}

fn command(input: &str) -> IResult<&str, &str> {
    preceded(space0, recognize((word, many0((space1, word))))).parse(input)
}

fn chain(input: &str) -> IResult<&str, Chain<'_>> {
    let tail = many0((preceded(space0, op), command));
    map((command, tail), |(head, tail)| Chain { head, tail }).parse(input)
}

#[test_case]
fn test_separator() {
    assert_eq!(separator(";"), Ok(("", ';')));
    assert_eq!(separator("; print foo"), Ok((" print foo", ';')));

    assert_eq!(separator("\n"), Ok(("", '\n')));
    assert_eq!(separator("\nprint foo"), Ok(("print foo", '\n')));
    assert_eq!(separator("# comment \nprint foo"), Ok(("print foo", '\n')));
    assert_eq!(separator("# bar; baz\nprint foo"), Ok(("print foo", '\n')));

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
    assert_eq!(substitution("$(print a)"), Ok(("", "$(print a)")));
    assert_eq!(substitution("$(print a # b)"), Ok(("", "$(print a # b)")));
    assert_eq!(substitution("$(print \"(a)\")"), Ok(("", "$(print \"(a)\")")));
}

#[test_case]
fn test_command() {
    assert_eq!(command("print foo"), Ok(("", "print foo")));
    assert_eq!(command("print foo; bar"), Ok(("; bar", "print foo")));
    assert_eq!(command("print foo # bar"), Ok((" # bar", "print foo")));
}

#[test_case]
fn test_parse() {
    use alloc::vec;

    let foo = Chain { head: "print foo", tail: vec![] };
    let bar = Chain { head: "bar", tail: vec![] };
    assert_eq!(parse("print foo; bar"), Ok(("", vec![foo, bar])));
    let chain = Chain {
        head: "print foo",
        tail: vec![(Op::And, "bar"), (Op::Or, "baz")],
    };
    assert_eq!(parse("print foo and bar or baz"), Ok(("", vec![chain])));
}
