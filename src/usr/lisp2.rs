use alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use nom::IResult;
use nom::branch::alt;
use nom::character::complete::{alpha1, alphanumeric0, char};
use nom::character::complete::multispace0;
use nom::character::complete::anychar;
use nom::character::is_space;
use nom::bytes::complete::take_till;
use nom::bytes::complete::is_not;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::number::complete::double;
use nom::sequence::delimited;
use nom::sequence::pair;

#[derive(Debug, PartialEq)]
pub enum Exp {
    Str(String),
    Sym(String),
    Num(f64),
    List(Vec<Exp>),
}

// Parser

fn parse_exp(input: &str) -> IResult<&str, Exp> {
    delimited(multispace0, alt((parse_num, parse_str, parse_list, parse_sym)), multispace0)(input)
}

fn parse_list(input: &str) -> IResult<&str, Exp> {
    let (input, list) = delimited(char('('), many0(parse_exp), char(')'))(input)?;
    Ok((input, Exp::List(list)))
}

fn parse_str(input: &str) -> IResult<&str, Exp> {
    let (input, s) = delimited(char('"'), is_not("\""), char('"'))(input)?;
    Ok((input, Exp::Str(s.to_string())))
}

fn parse_sym(input: &str) -> IResult<&str, Exp> {
    let (input, sym) = recognize(pair(alpha1, alphanumeric0))(input)?;
    Ok((input, Exp::Sym(sym.to_string())))
}

fn parse_num(input: &str) -> IResult<&str, Exp> {
    let (input, num) = double(input)?;
    Ok((input, Exp::Num(num)))
}

#[test_case]
fn test_lisp2() {
    macro_rules! eval {
        ($e:expr) => {
            format!("{:?}", parse_exp($e).unwrap())
        };
    }

    println!();
    println!("{}", eval!("abc1"));
    println!("{}", eval!("(abc1)"));
    println!("{}", eval!("(a)"));
    println!("{}", eval!("(a b)"));
    println!("{}", eval!("(a b c)"));
    println!("{}", eval!("(1)"));
    println!("{}", eval!("(1.23)"));
    println!("{}", eval!("(1 2 3)"));
    println!("{}", eval!("(print \"test\")"));
    assert!(true);
}
