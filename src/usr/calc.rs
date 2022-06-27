use crate::api::process;
use crate::api::prompt::Prompt;
use crate::api::console::Style;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nom::branch::alt;
use nom::character::complete::{char, space0};
use nom::number::complete::double;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::{delimited, tuple};
use nom::IResult;

// Adapted from Basic Calculator
// Copyright 2021 Balaji Sivaraman
// https://github.com/balajisivaraman/basic_calculator_rs

#[derive(Debug, PartialEq)]
pub enum Exp {
    Num(f64),
    Add(Box<Exp>, Box<Exp>),
    Sub(Box<Exp>, Box<Exp>),
    Mul(Box<Exp>, Box<Exp>),
    Div(Box<Exp>, Box<Exp>),
    Exp(Box<Exp>, Box<Exp>),
    Mod(Box<Exp>, Box<Exp>),
}

// Parser

fn parse(input: &str) -> IResult<&str, Exp> {
    let (input, num1) = parse_term(input)?;
    let (input, exps) = many0(tuple((alt((char('+'), char('-'))), parse_term)))(input)?;
    Ok((input, parse_exp(num1, exps)))
}

fn parse_term(input: &str) -> IResult<&str, Exp> {
    let (input, num1) = parse_factor(input)?;
    let (input, exps) = many0(tuple((alt((char('%'), char('/'), char('*'))), parse_factor)))(input)?;
    Ok((input, parse_exp(num1, exps)))
}

fn parse_factor(input: &str) -> IResult<&str, Exp> {
    let (input, num1) = alt((parse_parens, parse_num))(input)?;
    let (input, exps) = many0(tuple((char('^'), parse_factor)))(input)?;
    Ok((input, parse_exp(num1, exps)))
}

fn parse_parens(input: &str) -> IResult<&str, Exp> {
    delimited(space0, delimited(char('('), parse, char(')')), space0)(input)
}

fn parse_num(input: &str) -> IResult<&str, Exp> {
    map(delimited(space0, double, space0), Exp::Num)(input)
}

fn parse_exp(exp: Exp, rem: Vec<(char, Exp)>) -> Exp {
    rem.into_iter().fold(exp, |acc, val| parse_op(val, acc))
}

fn parse_op(tup: (char, Exp), exp1: Exp) -> Exp {
    let (op, exp2) = tup;
    match op {
        '+' => Exp::Add(Box::new(exp1), Box::new(exp2)),
        '-' => Exp::Sub(Box::new(exp1), Box::new(exp2)),
        '*' => Exp::Mul(Box::new(exp1), Box::new(exp2)),
        '/' => Exp::Div(Box::new(exp1), Box::new(exp2)),
        '^' => Exp::Exp(Box::new(exp1), Box::new(exp2)),
        '%' => Exp::Mod(Box::new(exp1), Box::new(exp2)),
        _ => panic!("Unknown operation"),
    }
}

// Evaluation

fn eval(exp: Exp) -> f64 {
    match exp {
        Exp::Num(num) => num,
        Exp::Add(exp1, exp2) => eval(*exp1) + eval(*exp2),
        Exp::Sub(exp1, exp2) => eval(*exp1) - eval(*exp2),
        Exp::Mul(exp1, exp2) => eval(*exp1) * eval(*exp2),
        Exp::Div(exp1, exp2) => eval(*exp1) / eval(*exp2),
        Exp::Exp(exp1, exp2) => libm::pow(eval(*exp1), eval(*exp2)),
        Exp::Mod(exp1, exp2) => libm::fmod(eval(*exp1), eval(*exp2)),
    }
}

// REPL

fn parse_eval(line: &str) -> Result<f64, String> {
    match parse(line) {
        Ok((line, parsed)) => {
            if line.is_empty() {
                Ok(eval(parsed))
            } else {
                Err(format!("Could not parse '{}'", line))
            }
        },
        Err(_) => {
            Err(format!("Could not parse '{}'", line))
        },
    }
}

fn repl() -> Result<(), usize> {
    println!("MOROS Calc v0.1.0\n");
    let csi_color = Style::color("Cyan");
    let csi_error = Style::color("LightRed");
    let csi_reset = Style::reset();
    let prompt_string = format!("{}>{} ", csi_color, csi_reset);

    let mut prompt = Prompt::new();
    let history_file = "~/.calc-history";
    prompt.history.load(history_file);

    while let Some(line) = prompt.input(&prompt_string) {
        if line == "q" || line == "quit" {
            break;
        }
        if line.is_empty() {
            println!();
            continue;
        }

        match parse_eval(&line) {
            Ok(res) => {
                println!("{}\n", res);
            }
            Err(msg) => {
                println!("{}Error:{} {}\n", csi_error, csi_reset, msg);
                continue;
            }
        }

        prompt.history.add(&line);
        prompt.history.save(history_file);
    }
    Ok(())
}

pub fn main(args: &[&str]) -> Result<(), usize> {
    if args.len() == 1 {
        repl()
    } else {
        match parse_eval(&args[1..].join(" ")) {
            Ok(res) => {
                println!("{}", res);
                Ok(())
            }
            Err(msg) => {
                error!("{}", msg);
                Err(process::EXIT_FAILURE)
            }
        }
    }
}

#[test_case]
fn test_calc() {
    macro_rules! eval {
        ($e:expr) => {
            format!("{}", parse_eval($e).unwrap())
        };
    }

    assert_eq!(eval!("1"),                       "1");
    assert_eq!(eval!("1.5"),                   "1.5");

    assert_eq!(eval!("+1"),                      "1");
    assert_eq!(eval!("-1"),                     "-1");

    assert_eq!(eval!("1 + 2"),                   "3");
    assert_eq!(eval!("1 + 2 + 3"),               "6");
    assert_eq!(eval!("1 + 2.5"),               "3.5");
    assert_eq!(eval!("1 + 2.5"),               "3.5");
    assert_eq!(eval!("2 - 1"),                   "1");
    assert_eq!(eval!("1 - 2"),                  "-1");
    assert_eq!(eval!("2 * 3"),                   "6");
    assert_eq!(eval!("2 * 3.5"),                 "7");
    assert_eq!(eval!("6 / 2"),                   "3");
    assert_eq!(eval!("6 / 4"),                 "1.5");
    assert_eq!(eval!("2 ^ 4"),                  "16");
    assert_eq!(eval!("3 % 2"),                   "1");

    assert_eq!(eval!("2 * 3 + 4"),              "10");
    assert_eq!(eval!("2 * (3 + 4)"),            "14");
    assert_eq!(eval!("2 ^ 4 + 1"),              "17");
    assert_eq!(eval!("1 + 2 ^ 4"),              "17");
    assert_eq!(eval!("1 + 3 * 2 ^ 4 * 2 + 3"), "100");
}
