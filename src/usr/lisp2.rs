use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;

use nom::IResult;
use nom::branch::alt;
use nom::character::complete::char;
use nom::character::complete::multispace0;
use nom::bytes::complete::take_while1;
use nom::bytes::complete::is_not;
use nom::sequence::preceded;
use nom::multi::many0;
use nom::number::complete::double;
use nom::sequence::delimited;

#[derive(Debug, PartialEq)]
pub enum Exp {
    Str(String),
    Sym(String),
    Num(f64),
    List(Vec<Exp>),
}

// Parse

fn is_symbol_letter(c: char) -> bool {
    let chars = "-+*/";
    c.is_alphanumeric() || chars.contains(c)
}

fn parse_exp(input: &str) -> IResult<&str, Exp> {
    delimited(multispace0, alt((parse_num, parse_str, parse_list, parse_quote, parse_sym)), multispace0)(input)
}

fn parse_list(input: &str) -> IResult<&str, Exp> {
    let (input, list) = delimited(char('('), many0(parse_exp), char(')'))(input)?;
    Ok((input, Exp::List(list)))
}

fn parse_quote(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(char('\''), parse_exp)(input)?;
    let list = vec![Exp::Sym("quote".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_str(input: &str) -> IResult<&str, Exp> {
    let (input, s) = delimited(char('"'), is_not("\""), char('"'))(input)?;
    Ok((input, Exp::Str(s.to_string())))
}

fn parse_sym(input: &str) -> IResult<&str, Exp> {
    let (input, sym) = take_while1(is_symbol_letter)(input)?;
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
    println!("{}", eval!("(a-b-c)"));
    println!("{}", eval!("(+ 1 2 3)"));
    println!("{}", eval!("(print \"test\")"));
    println!("{}", eval!("'1"));
    println!("{}", eval!("'(1 2 3)"));
    assert!(true);
}
