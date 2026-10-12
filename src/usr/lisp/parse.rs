use super::{Err, Exp, Number};
use crate::could_not;
use crate::api::parse::{ignored, hexadecimal, decimal, binary, float, string};

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use nom::Err::Error;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::character::complete::char;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;

fn is_symbol_letter(c: char) -> bool {
    let chars = "$<>=-+*/%^?!.&|";
    c.is_alphanumeric() || chars.contains(c)
}

fn parse_str(input: &str) -> IResult<&str, Exp> {
    map(string, Exp::Str).parse(input)
}

fn parse_sym(input: &str) -> IResult<&str, Exp> {
    let (input, sym) = take_while1(is_symbol_letter).parse(input)?;
    Ok((input, Exp::Sym(sym.to_string())))
}

fn parse_num(input: &str) -> IResult<&str, Exp> {
    let (input, num) = recognize((
        opt(alt((char('+'), char('-')))),
        alt((float, hexadecimal, binary, decimal))
    )).parse(input)?;
    Ok((input, Exp::Num(Number::from(num))))
}

fn parse_bool(input: &str) -> IResult<&str, Exp> {
    let (input, s) = alt((tag("true"), tag("false"))).parse(input)?;
    Ok((input, Exp::Bool(s == "true")))
}

fn parse_list(input: &str) -> IResult<&str, Exp> {
    let (input, list) = delimited(
        char('('), many0(parse_exp), char(')')
    ).parse(input)?;
    Ok((input, Exp::List(list)))
}

fn parse_quote(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(char('\''), parse_exp).parse(input)?;
    let list = vec![Exp::Sym("quote".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_unquote_splice(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(tag(",@"), parse_exp).parse(input)?;
    let list = vec![Exp::Sym("unquote-splice".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_splice(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(tag("@"), parse_exp).parse(input)?;
    let list = vec![Exp::Sym("splice".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_unquote(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(char(','), parse_exp).parse(input)?;
    let list = vec![Exp::Sym("unquote".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_quasiquote(input: &str) -> IResult<&str, Exp> {
    let (input, list) = preceded(char('`'), parse_exp).parse(input)?;
    let list = vec![Exp::Sym("quasiquote".to_string()), list];
    Ok((input, Exp::List(list)))
}

fn parse_exp(input: &str) -> IResult<&str, Exp> {
    preceded(
        ignored,
        alt((
            parse_num,
            parse_bool,
            parse_str,
            parse_list,
            parse_quote,
            parse_quasiquote,
            parse_unquote_splice,
            parse_unquote,
            parse_splice,
            parse_sym
        ))
    ).parse(input)
}

pub fn parse(input: &str) -> Result<(String, Exp), Err> {
    match parse_exp(input) {
        Ok((input, exp)) => Ok((input.to_string(), exp)),
        Err(Error(err)) => {
            if err.input.is_empty() {
                Ok(("".to_string(), Exp::List(vec![
                    Exp::Sym("quote".to_string()),
                    Exp::List(vec![])
                ])))
            } else {
                let line = err.input.lines().next().unwrap();
                could_not!("parse {:?}", line)
            }
        }
        _ => could_not!("parse input"),
    }
}
