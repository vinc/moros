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
use nom::combinator::recognize;
use nom::combinator::value;
use nom::error::Error;
use nom::multi::many0;
use nom::multi::many1;
use nom::multi::separated_list0;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::terminated;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    And,
    Or
}

#[derive(Debug, PartialEq)]
pub enum Exp<'a> {
    Chain(Chain<'a>),
    If(Chain<'a>, Vec<Exp<'a>>, Vec<Exp<'a>>)
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

fn keyword<'a>(
    name: &'static str
) -> impl Parser<&'a str, Output = &'a str, Error = Error<&'a str>> {
    terminated(tag(name), not(part))
}

fn op(input: &str) -> IResult<&str, Op> {
    alt((
        value(Op::And, keyword("and")),
        value(Op::Or, keyword("or"))
    )).parse(input)
}

fn text(input: &str) -> IResult<&str, &str> {
    is_not(" \t\"$;#\n").parse(input)
}

fn part(input: &str) -> IResult<&str, &str> {
    alt((quoted, substitution, tag("$"), text)).parse(input)
}

fn command(input: &str) -> IResult<&str, &str> {
    preceded(space0, recognize((word, many0((space1, word))))).parse(input)
}

fn chain(input: &str) -> IResult<&str, Chain<'_>> {
    let tail = many0((preceded(space0, op), command));
    map((command, tail), |(head, tail)| Chain { head, tail }).parse(input)
}

fn reserved(input: &str) -> IResult<&str, &str> {
    alt((
        keyword("and"),
        keyword("do"),
        keyword("else"),
        keyword("end"),
        keyword("if"),
        keyword("or"),
    )).parse(input)
}

fn word(input: &str) -> IResult<&str, &str> {
    preceded(not(reserved), recognize(many1(part))).parse(input)
}

fn if_exp(input: &str) -> IResult<&str, Exp<'_>> {
    let (input, _) = preceded(space0, keyword("if")).parse(input)?;
    let (input, cond) = chain(input)?;
    let (input, _) = alt((
        value((), separator),
        value((), preceded(space0, keyword("do"))),
    )).parse(input)?;
    let (input, then) = body(input)?;
    let (input, other) = opt(
        preceded(preceded(space0, keyword("else")), body)
    ).parse(input)?;
    let (input, _) = preceded(space0, keyword("end")).parse(input)?;
    Ok((input, Exp::If(cond, then, other.unwrap_or_default())))
}

fn exp(input: &str) -> IResult<&str, Exp<'_>> {
    alt((if_exp, map(chain, Exp::Chain))).parse(input)
}

fn body(input: &str) -> IResult<&str, Vec<Exp<'_>>> {
    delimited(
        many0(separator),
        separated_list0(many1(separator), exp),
        many0(separator),
    ).parse(input)
}

pub fn parse(input: &str) -> IResult<&str, Vec<Exp<'_>>> {
    terminated(body, (blank, eof)).parse(input)
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

    let chain = |head| Chain { head, tail: vec![] };
    let cmd = |head| Exp::Chain(chain(head));

    assert_eq!(parse("print foo; bar"), Ok(("", vec![
        cmd("print foo"), cmd("bar")
    ])));

    assert_eq!(parse("print foo and bar or baz"), Ok(("", vec![
        Exp::Chain(Chain {
             head: "print foo",
             tail: vec![(Op::And, "bar"), (Op::Or, "baz")],
        })
    ])));

    assert_eq!(parse("if foo do bar else baz end"), Ok(("", vec![
        Exp::If(chain("foo"),
        vec![cmd("bar")],
        vec![cmd("baz")])
    ])));

    assert_eq!(parse("if foo\n  bar\nelse\n  baz\nend\n"), Ok(("", vec![
        Exp::If(chain("foo"),
        vec![cmd("bar")],
        vec![cmd("baz")])
    ])));

    assert_eq!(parse("if foo; bar; end"), Ok(("", vec![
        Exp::If(chain("foo"),
        vec![cmd("bar")],
        vec![])
    ])));

    assert_eq!(parse("if foo do if bar do baz end else qux end"), Ok(("", vec![
        Exp::If(chain("foo"),
        vec![Exp::If(chain("bar"), vec![cmd("baz")], vec![])],
        vec![cmd("qux")])
    ])));

    assert_eq!(parse("print \"end\""), Ok(("", vec![
        cmd("print \"end\"")
    ])));

    assert!(parse("print end").is_err());
    assert!(parse("if foo do bar").is_err());
    assert!(parse("if foo do bar end baz").is_err());
    assert!(parse("end").is_err());
}
