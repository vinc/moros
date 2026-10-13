use alloc::format;
use alloc::string::{String, ToString};
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::escaped_transform;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::character::complete::multispace1;
use nom::character::complete::not_line_ending;
use nom::character::complete::one_of;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::combinator::value;
use nom::multi::many0;
use nom::multi::many0_count;
use nom::multi::many1;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::terminated;

fn comment(input: &str) -> IResult<&str, &str> {
    preceded(char('#'), not_line_ending).parse(input)
}

// Whitespaces and comments are ignored
pub fn ignored(input: &str) -> IResult<&str, ()> {
    map(many0_count(alt((multispace1, comment))), |_| ()).parse(input)
}

// https://docs.rs/nom/latest/nom/recipes/index.html#hexadecimal
fn hexadecimal(input: &str) -> IResult<&str, &str> {
    preceded(
        tag("0x"),
        recognize(many1(terminated(
            one_of("0123456789abcdefABCDEF"),
            many0(char('_'))
        )))
    ).parse(input)
}

// https://docs.rs/nom/latest/nom/recipes/index.html#decimal
fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(
        many1(terminated(one_of("0123456789"), many0(char('_'))))
    ).parse(input)
}

// https://docs.rs/nom/latest/nom/recipes/index.html#binary
fn binary(input: &str) -> IResult<&str, &str> {
    preceded(
        tag("0b"),
        recognize(many1(terminated(one_of("01"), many0(char('_')))))
    ).parse(input)
}

// https://docs.rs/nom/latest/nom/recipes/index.html#floating-point-numbers
fn float(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(
            // .42
            (
                char('.'),
                decimal,
                opt((one_of("eE"), opt(one_of("+-")), decimal))
            )
        ),
        recognize(
            // 42e42 and 42.42e42
            (
                decimal,
                opt(preceded(char('.'), decimal)),
                one_of("eE"),
                opt(one_of("+-")),
                decimal
            )
        ),
        recognize(
            // 42. and 42.42
            (decimal, char('.'), opt(decimal))
        )
    )).parse(input)
}

pub fn number(input: &str) -> IResult<&str, &str> {
    recognize((
        opt(one_of("+-")),
        alt((float, hexadecimal, binary, decimal))
    )).parse(input)
}

pub fn string(input: &str) -> IResult<&str, String> {
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

pub fn quote(s: &str) -> String {
    format!("{:?}", s).replace("\\u{8}", "\\b").replace("\\u{1b}", "\\e")
}

#[test_case]
fn test_quote() {
    let strings = [
        "",
        "  spaced  ",
        "print \"hello\"",
        "/bin/sleep",
        "/tmp/a\\b",
        "line 1\nline 2\r\n",
        "\x1B[1mbold\x1B[0m\t\x08",
    ];
    for s in strings {
        assert_eq!(string(&quote(s)), Ok(("", s.to_string())));
    }
}
