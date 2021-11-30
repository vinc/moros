use crate::usr;
use crate::api::console::Style;
use crate::api::prompt::Prompt;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;
use core::fmt;

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

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Exp::Str(s) => s.clone(),
            Exp::Sym(s) => s.clone(),
            Exp::Num(n) => n.to_string(),
            Exp::List(list) => {
                let xs: Vec<String> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(" "))
            },
        };
        write!(f, "{}", s)
    }
}

// Parse

fn is_symbol_letter(c: char) -> bool {
    let chars = "-+*/";
    c.is_alphanumeric() || chars.contains(c)
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

fn parse(input: &str) -> Result<Exp, ()> {
    if let Ok((_, exp)) = parse_exp(input) {
        Ok(exp)
    } else {
        Err(())
    }
}

// Eval

fn eval_builtin(fun: &Exp, args: &[Exp]) -> Exp {
    match fun {
        Exp::Sym(sym) => {
            let first = eval_num(&args[0]);
            let rest = &args[1..];
            match sym.as_ref() {
                "+" => Exp::Num(rest.iter().fold(first, |acc, num| acc + eval_num(num))),
                "-" => Exp::Num(rest.iter().fold(first, |acc, num| acc - eval_num(num))),
                "*" => Exp::Num(rest.iter().fold(first, |acc, num| acc * eval_num(num))),
                _   => panic!("eval_builtin: not implemented"),
            }
        }
        _ => {
            panic!("eval_builtin: not implemented");
        }
    }
}

fn eval_num(exp: &Exp) -> f64 {
    match exp {
        Exp::Num(num) => {
            *num
        }
        _ => {
            panic!("eval_num: not implemented");
        }
    }
}

fn eval(exp: Exp) -> Exp {
    match exp {
        Exp::List(list) => {
            let fun = &list[0];
            let args = &list[1..];
            eval_builtin(fun, args)
        }
        _ => {
            exp
        }
    }
}

// REPL

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    println!("MOROS Lisp v0.1.0\n");

    let csi_color = Style::color("Cyan");
    let csi_error = Style::color("LightRed");
    let csi_reset = Style::reset();
    let prompt_string = format!("{}>{} ", csi_color, csi_reset);

    let mut prompt = Prompt::new();
    let history_file = "~/.lisp-history";
    prompt.history.load(history_file);
    //prompt.completion.set(&lisp_completer);

    while let Some(line) = prompt.input(&prompt_string) {
        if line == "(exit)" || line == "(quit)" {
            break;
        }
        if line.is_empty() {
            println!();
            continue;
        }
        match parse(&line) {
            Ok(exp) => {
                println!("{}\n", eval(exp));
            }
            Err(err) => {
                println!("{}Error:{} {:?}\n", csi_error, csi_reset, err);
            }
        }
        prompt.history.add(&line);
        prompt.history.save(history_file);
    }
    usr::shell::ExitCode::CommandSuccessful
}

#[test_case]
fn test_lisp2() {
    macro_rules! eval {
        ($e:expr) => {
            format!("{}", eval(parse($e).unwrap()))
        };
    }

    assert_eq!(eval!("(+ 1 2 3)"), "6");
    assert_eq!(eval!("(- 3 2)"), "1");
    assert_eq!(eval!("(* 2 3 4)"), "24");
}
