use crate::{api, usr};
use crate::api::fs;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::prompt::Prompt;
use crate::api::regex::Regex;

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::convert::TryInto;
use core::convert::TryFrom;
use core::f64::consts::PI;
use core::fmt;
use core::ops::{Neg, Add, Div, Mul, Sub, Rem};
use core::str::FromStr;
use float_cmp::approx_eq;
use lazy_static::lazy_static;
use num_bigint::BigInt;
use num_traits::cast::ToPrimitive;
use spin::Mutex;

use nom::IResult;
use nom::branch::alt;
use nom::bytes::complete::escaped_transform;
use nom::bytes::complete::is_not;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::character::complete::char;
use nom::character::complete::multispace0;
use nom::character::complete::digit1;
use nom::combinator::map;
use nom::combinator::opt;
use nom::combinator::value;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::tuple;
use nom::combinator::recognize;

// Eval & Env adapted from Risp
// Copyright 2019 Stepan Parunashvili
// https://github.com/stopachka/risp
//
// Parser rewritten from scratch using Nom
// https://github.com/geal/nom
//
// See "Recursive Functions of Symic Expressions and Their Computation by Machine" by John McCarthy (1960)
// And "The Roots of Lisp" by Paul Graham (2002)
//
// MOROS Lisp is a lisp-1 like Scheme and Clojure
// See "Technical Issues of Separation in Function Cells and Value Cells" by Richard P. Gabriel (1982)

// Types

#[derive(Clone, PartialEq)]
enum Number {
    BigInt(BigInt),
    Float(f64),
    Int(i64),
}

impl Number {
    fn cos(&self) -> Number {
        Number::Float(libm::cos(self.into()))
    }

    fn sin(&self) -> Number {
        Number::Float(libm::sin(self.into()))
    }

    fn tan(&self) -> Number {
        Number::Float(libm::tan(self.into()))
    }

    fn acos(&self) -> Number {
        Number::Float(libm::acos(self.into()))
    }

    fn asin(&self) -> Number {
        Number::Float(libm::asin(self.into()))
    }

    fn atan(&self) -> Number {
        Number::Float(libm::atan(self.into()))
    }

    fn pow(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::BigInt(a), Number::Int(b))   => Number::BigInt(a.pow(*b as u32)), // FIXME
            (Number::Int(a),    Number::Int(b))   => {
                if let Some(r) = a.checked_pow(*b as u32) {
                    Number::Int(r)
                } else {
                    Number::BigInt(BigInt::from(*a)).pow(other)
                }
            }
            (Number::Int(a),    Number::Float(b)) => Number::Float(libm::pow(*a as f64, *b)),
            (Number::Float(a),  Number::Int(b))   => Number::Float(libm::pow(*a, *b as f64)),
            (Number::Float(a),  Number::Float(b)) => Number::Float(libm::pow(*a, *b)),
            _                                     => Number::Float(f64::NAN), // TODO
        }
    }
}

impl Neg for Number {
    type Output = Number;

    fn neg(self) -> Number {
        match self {
            Number::BigInt(a) => Number::BigInt(-a),
            Number::Int(a)    => {
                if let Some(r) = a.checked_neg() {
                    Number::Int(r)
                } else {
                    Number::BigInt(-BigInt::from(a))
                }
            }
            Number::Float(a)  => Number::Float(-a),
        }
    }
}

impl Add for Number {
    type Output = Number;

    fn add(self, other: Number) -> Number {
        match (self, other) {
            (Number::BigInt(a), Number::BigInt(b)) => Number::BigInt(a + b),
            (Number::BigInt(a), Number::Int(b))    => Number::BigInt(a + b),
            (Number::Int(a),    Number::BigInt(b)) => Number::BigInt(a + b),
            (Number::Int(a),    Number::Int(b))    => {
                if let Some(r) = a.checked_add(b) {
                    Number::Int(r)
                } else {
                    Number::BigInt(BigInt::from(a) + BigInt::from(b))
                }
            }
            (Number::Int(a),    Number::Float(b))  => Number::Float((a as f64) + b),
            (Number::Float(a),  Number::Int(b))    => Number::Float(a + (b as f64)),
            (Number::Float(a),  Number::Float(b))  => Number::Float(a + b),
            _                                      => Number::Float(f64::NAN), // TODO
        }
    }
}

impl Div for Number {
    type Output = Number;

    fn div(self, other: Number) -> Number {
        match (self, other) {
            (Number::BigInt(a), Number::BigInt(b)) => Number::BigInt(a / b),
            (Number::BigInt(a), Number::Int(b))    => Number::BigInt(a / b),
            (Number::Int(a),    Number::BigInt(b)) => Number::BigInt(a / b),
            (Number::Int(a),    Number::Int(b))    => {
                if let Some(r) = a.checked_div(b) {
                    Number::Int(r)
                } else {
                    Number::BigInt(BigInt::from(a) / BigInt::from(b))
                }
            }
            (Number::Int(a),    Number::Float(b))  => Number::Float((a as f64) / b),
            (Number::Float(a),  Number::Int(b))    => Number::Float(a / (b as f64)),
            (Number::Float(a),  Number::Float(b))  => Number::Float(a / b),
            _                                      => Number::Float(f64::NAN), // TODO
        }
    }
}

impl Mul for Number {
    type Output = Number;

    fn mul(self, other: Number) -> Number {
        match (self, other) {
            (Number::BigInt(a), Number::BigInt(b)) => Number::BigInt(a * b),
            (Number::BigInt(a), Number::Int(b))    => Number::BigInt(a * b),
            (Number::Int(a),    Number::BigInt(b)) => Number::BigInt(a * b),
            (Number::Int(a),    Number::Int(b))    => {
                if let Some(r) = a.checked_mul(b) {
                    Number::Int(r)
                } else {
                    Number::BigInt(BigInt::from(a) * BigInt::from(b))
                }
            }
            (Number::Int(a),    Number::Float(b))  => Number::Float((a as f64) * b),
            (Number::Float(a),  Number::Int(b))    => Number::Float(a * (b as f64)),
            (Number::Float(a),  Number::Float(b))  => Number::Float(a * b),
            _                                      => Number::Float(f64::NAN), // TODO
        }
    }
}

impl Sub for Number {
    type Output = Number;

    fn sub(self, other: Number) -> Number {
        match (self, other) {
            (Number::BigInt(a), Number::BigInt(b)) => Number::BigInt(a - b),
            (Number::BigInt(a), Number::Int(b))    => Number::BigInt(a - b),
            (Number::Int(a),    Number::BigInt(b)) => Number::BigInt(a - b),
            (Number::Int(a),    Number::Int(b))    => {
                if let Some(r) = a.checked_div(b) {
                    Number::Int(r)
                } else {
                    Number::BigInt(BigInt::from(a) - BigInt::from(b))
                }
            }
            (Number::Int(a),    Number::Float(b))  => Number::Float((a as f64) - b),
            (Number::Float(a),  Number::Int(b))    => Number::Float(a - (b as f64)),
            (Number::Float(a),  Number::Float(b))  => Number::Float(a - b),
            _                                      => Number::Float(f64::NAN), // TODO
        }
    }
}

impl Rem for Number {
    type Output = Number;

    fn rem(self, other: Number) -> Number {
        match (self, other) {
            (Number::BigInt(a), Number::BigInt(b)) => Number::BigInt(a % b),
            (Number::BigInt(a), Number::Int(b))    => Number::BigInt(a % b),
            (Number::Int(a),    Number::BigInt(b)) => Number::BigInt(a % b),
            (Number::Int(a),    Number::Int(b))    => {
                if let Some(r) = a.checked_rem(b) {
                    Number::Int(r)
                } else {
                    Number::BigInt(BigInt::from(a) % BigInt::from(b))
                }
            }
            (Number::Int(a),    Number::Float(b))  => Number::Float(libm::fmod(a as f64, b)),
            (Number::Float(a),  Number::Int(b))    => Number::Float(libm::fmod(a, b as f64)),
            (Number::Float(a),  Number::Float(b))  => Number::Float(libm::fmod(a, b)),
            _                                      => Number::Float(f64::NAN), // TODO
        }
    }
}

impl FromStr for Number {
    type Err = Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains('.') {
            if let Ok(n) = s.parse() {
                return Ok(Number::Float(n));
            }
        } else if let Ok(n) = s.parse() {
            return Ok(Number::Int(n));
        } else {
            let mut chars = s.chars().peekable();
            let is_neg = chars.peek() == Some(&&'-');
            if is_neg {
                chars.next().unwrap();
            }
            let mut res = BigInt::from(0);
            for c in chars {
                let d = c as u8 - b'0';
                res = res * BigInt::from(10) + BigInt::from(d as u32);
            }
            res *= BigInt::from(if is_neg { -1 } else { 1 });
            return Ok(Number::BigInt(res));
        } /* else if let Ok(n) = s.parse() { // FIXME: rust-lld: error: undefined symbol: fmod
            return Ok(Number::BigInt(n));
        } */
        Err(Err::Reason("Could not parse number".to_string()))
    }
}

impl From<&str> for Number {
    fn from(s: &str) -> Self {
        if let Ok(num) = s.parse() {
            num
        } else {
            Number::Float(f64::NAN)
        }
    }
}

impl From<f64> for Number {
    fn from(num: f64) -> Self {
        Number::Float(num)
    }
}

impl From<usize> for Number {
    fn from(num: usize) -> Self {
        if num > i64::MAX as usize {
            Number::BigInt(BigInt::from(num))
        } else {
            Number::Int(num as i64)
        }
    }
}

impl From<u8> for Number {
    fn from(num: u8) -> Self {
        Number::Int(num as i64)
    }
}

impl From<&Number> for f64 {
    fn from(num: &Number) -> f64 {
        match num {
            Number::Float(n)  => *n,
            Number::Int(n)    => *n as f64,
            Number::BigInt(_) => f64::INFINITY, // TODO
        }
    }
}

impl TryFrom<Number> for u8 {
    type Error = Err;

    fn try_from(num: Number) -> Result<Self, Self::Error> {
        let num = f64::from(&num);
        if num >= 0.0 && num < u8::MAX.into() && (num - libm::trunc(num) == 0.0) {
            Ok(num as u8)
        } else {
            Err(Err::Reason(format!("Expected an integer between 0 and {}", u8::MAX)))
        }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //write!(f, "{}", self), // FIXME: alloc error
        match self {
            Number::Int(n) => {
                write!(f, "{}", n)
            }
            Number::BigInt(n) => {
                //write!(f, "{}", n), // FIXME: rust-lld: error: undefined symbol: fmod
                let mut v = Vec::new();
                let mut n = n.clone();
                if n < BigInt::from(0) {
                    write!(f, "-").ok();
                    n = -n;
                }
                loop {
                    v.push((n.clone() % BigInt::from(10)).to_u64().unwrap());
                    n = n / BigInt::from(10);
                    if n == BigInt::from(0) {
                        break;
                    }
                }
                for d in v.iter().rev() {
                    write!(f, "{}", d).ok();
                }
                Ok(())
            }
            Number::Float(n) => {
                if n - libm::trunc(*n) == 0.0 {
                    write!(f, "{}.0", n)
                } else {
                    write!(f, "{}", n)
                }
            }
        }
    }
}

#[derive(Clone)]
enum Exp {
    Primitive(fn(&[Exp]) -> Result<Exp, Err>),
    Lambda(Lambda),
    List(Vec<Exp>),
    Bool(bool),
    Num(Number),
    Str(String),
    Sym(String),
}

impl PartialEq for Exp {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Exp::Lambda(a), Exp::Lambda(b)) => a == b,
            (Exp::List(a),   Exp::List(b))   => a == b,
            (Exp::Bool(a),   Exp::Bool(b))   => a == b,
            (Exp::Num(a),    Exp::Num(b))    => a == b,
            (Exp::Str(a),    Exp::Str(b))    => a == b,
            (Exp::Sym(a),    Exp::Sym(b))    => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            Exp::Primitive(_) => "<function>".to_string(),
            Exp::Lambda(_)    => "<function>".to_string(),
            Exp::Bool(a)      => a.to_string(),
            Exp::Num(n)       => n.to_string(),
            Exp::Sym(s)       => s.clone(),
            Exp::Str(s)       => format!("{:?}", s),
            Exp::List(list)   => {
                let xs: Vec<String> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(" "))
            },
        };
        write!(f, "{}", out)
    }
}

#[derive(Clone, PartialEq)]
struct Lambda {
    params: Rc<Exp>,
    body: Rc<Exp>,
}

#[derive(Debug)]
enum Err {
    Reason(String),
}

#[derive(Clone)]
struct Env {
    data: BTreeMap<String, Exp>,
    outer: Option<Rc<RefCell<Env>>>,
}

lazy_static! {
    pub static ref FORMS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

fn lisp_completer(line: &str) -> Vec<String> {
    let mut entries = Vec::new();
    if let Some(last_word) = line.split_whitespace().next_back() {
        if let Some(f) = last_word.strip_prefix('(') {
            for form in &*FORMS.lock() {
                if let Some(entry) = form.strip_prefix(f) {
                    entries.push(entry.into());
                }
            }
        }
    }
    entries
}

// Parser

fn is_symbol_letter(c: char) -> bool {
    let chars = "<>=-+*/%^?:";
    c.is_alphanumeric() || chars.contains(c)
}

fn parse_str(input: &str) -> IResult<&str, Exp> {
    let escaped = map(opt(escaped_transform(is_not("\\\""), '\\', alt((
        value("\\", tag("\\")),
        value("\"", tag("\"")),
        value("\n", tag("n")),
    )))), |inner| inner.unwrap_or("".to_string()));
    let (input, s) = delimited(char('"'), escaped, char('"'))(input)?;
    Ok((input, Exp::Str(s)))
}

fn parse_sym(input: &str) -> IResult<&str, Exp> {
    let (input, sym) = take_while1(is_symbol_letter)(input)?;
    Ok((input, Exp::Sym(sym.to_string())))
}

fn parse_num(input: &str) -> IResult<&str, Exp> {
    let (input, num) = recognize(tuple((
        opt(alt((char('+'), char('-')))),
        digit1,
        opt(tuple((char('.'), digit1)))
    )))(input)?;
    Ok((input, Exp::Num(Number::from(num))))
}

fn parse_bool(input: &str) -> IResult<&str, Exp> {
    let (input, s) = alt((tag("true"), tag("false")))(input)?;
    Ok((input, Exp::Bool(s == "true")))
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

fn parse_exp(input: &str) -> IResult<&str, Exp> {
    delimited(multispace0, alt((parse_num, parse_bool, parse_str, parse_list, parse_quote, parse_sym)), multispace0)(input)
}

fn parse(input: &str)-> Result<(String, Exp), Err> {
    match parse_exp(input) {
        Ok((input, exp)) => Ok((input.to_string(), exp)),
        Err(_) => Err(Err::Reason("Could not parse input".to_string())),
    }
}

// Env

macro_rules! ensure_tonicity {
    ($check_fn:expr) => {
        |args: &[Exp]| -> Result<Exp, Err> {
            let floats = list_of_floats(args)?;
            ensure_length_gt!(floats, 0);
            let first = &floats[0];
            let rest = &floats[1..];
            fn f(prev: &f64, xs: &[f64]) -> bool {
                match xs.first() {
                    Some(x) => $check_fn(*prev, *x) && f(x, &xs[1..]),
                    None => true,
                }
            }
            Ok(Exp::Bool(f(first, rest)))
        }
    };
}

macro_rules! ensure_length_eq {
    ($list:expr, $count:expr) => {
        if $list.len() != $count {
            let plural = if $count != 1 { "s" } else { "" };
            return Err(Err::Reason(format!("Expected {} expression{}", $count, plural)))
        }
    };
}

macro_rules! ensure_length_gt {
    ($list:expr, $count:expr) => {
        if $list.len() <= $count {
            let plural = if $count != 1 { "s" } else { "" };
            return Err(Err::Reason(format!("Expected more than {} expression{}", $count, plural)))
        }
    };
}

fn default_env() -> Rc<RefCell<Env>> {
    let mut data: BTreeMap<String, Exp> = BTreeMap::new();
    data.insert("pi".to_string(), Exp::Num(Number::from(PI)));
    data.insert("=".to_string(), Exp::Primitive(ensure_tonicity!(|a, b| approx_eq!(f64, a, b))));
    data.insert(">".to_string(), Exp::Primitive(ensure_tonicity!(|a, b| !approx_eq!(f64, a, b) && a > b)));
    data.insert(">=".to_string(), Exp::Primitive(ensure_tonicity!(|a, b| approx_eq!(f64, a, b) || a > b)));
    data.insert("<".to_string(), Exp::Primitive(ensure_tonicity!(|a, b| !approx_eq!(f64, a, b) && a < b)));
    data.insert("<=".to_string(), Exp::Primitive(ensure_tonicity!(|a, b| approx_eq!(f64, a, b) || a < b)));
    data.insert("*".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        let res = list_of_numbers(args)?.iter().fold(Number::Int(1), |acc, a| acc * a.clone());
        Ok(Exp::Num(res))
    }));
    data.insert("+".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        let res = list_of_numbers(args)?.iter().fold(Number::Int(0), |acc, a| acc + a.clone());
        Ok(Exp::Num(res))
    }));
    data.insert("-".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_gt!(args, 0);
        let args = list_of_numbers(args)?;
        let car = args[0].clone();
        if args.len() == 1 {
            Ok(Exp::Num(-car))
        } else {
            let res = args[1..].iter().fold(Number::Int(0), |acc, a| acc + a.clone());
            Ok(Exp::Num(car - res))
        }
    }));
    data.insert("/".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_gt!(args, 0);
        let args = list_of_numbers(args)?;
        let car = args[0].clone();
        if args.len() == 1 {
            Ok(Exp::Num(Number::Int(1) / car))
        } else {
            let res = args[1..].iter().fold(car, |acc, a| acc / a.clone());
            Ok(Exp::Num(res))
        }
    }));
    data.insert("%".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_gt!(args, 0);
        let args = list_of_numbers(args)?;
        let car = args[0].clone();
        let res = args[1..].iter().fold(car, |acc, a| acc % a.clone());
        Ok(Exp::Num(res))
    }));
    data.insert("^".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_gt!(args, 0);
        let args = list_of_numbers(args)?;
        let car = args[0].clone();
        let res = args[1..].iter().fold(car, |acc, a| acc.pow(&a));
        Ok(Exp::Num(res))
    }));
    data.insert("cos".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        Ok(Exp::Num(number(&args[0])?.cos()))
    }));
    data.insert("acos".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        if -1.0 <= float(&args[0])? && float(&args[0])? <= 1.0 {
            Ok(Exp::Num(number(&args[0])?.acos()))
        } else {
            Err(Err::Reason("Expected arg to be between -1.0 and 1.0".to_string()))
        }
    }));
    data.insert("asin".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        if -1.0 <= float(&args[0])? && float(&args[0])? <= 1.0 {
            Ok(Exp::Num(number(&args[0])?.asin()))
        } else {
            Err(Err::Reason("Expected arg to be between -1.0 and 1.0".to_string()))
        }
    }));
    data.insert("atan".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        Ok(Exp::Num(number(&args[0])?.atan()))
    }));
    data.insert("sin".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        Ok(Exp::Num(number(&args[0])?.sin()))
    }));
    data.insert("tan".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        Ok(Exp::Num(number(&args[0])?.tan()))
    }));
    data.insert("system".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let cmd = string(&args[0])?;
        match usr::shell::exec(&cmd) {
            Ok(()) => Ok(Exp::Num(Number::from(0 as u8))),
            Err(code) => Ok(Exp::Num(Number::from(code as u8))),
        }
    }));
    data.insert("read-file".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let path = string(&args[0])?;
        let contents = fs::read_to_string(&path).or(Err(Err::Reason("Could not read file".to_string())))?;
        Ok(Exp::Str(contents))
    }));
    data.insert("read-file-bytes".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 2);
        let path = string(&args[0])?;
        let len = float(&args[1])?;
        let mut buf = vec![0; len as usize];
        let bytes = fs::read(&path, &mut buf).or(Err(Err::Reason("Could not read file".to_string())))?;
        buf.resize(bytes, 0);
        Ok(Exp::List(buf.iter().map(|b| Exp::Num(Number::from(*b))).collect()))
    }));
    data.insert("write-file-bytes".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 2);
        let path = string(&args[0])?;
        match &args[1] {
            Exp::List(list) => {
                let buf = list_of_bytes(list)?;
                let bytes = fs::write(&path, &buf).or(Err(Err::Reason("Could not write file".to_string())))?;
                Ok(Exp::Num(Number::from(bytes)))
            }
            _ => Err(Err::Reason("Expected second arg to be a list".to_string()))
        }
    }));
    data.insert("append-file-bytes".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 2);
        let path = string(&args[0])?;
        match &args[1] {
            Exp::List(list) => {
                let buf = list_of_bytes(list)?;
                let bytes = fs::append(&path, &buf).or(Err(Err::Reason("Could not write file".to_string())))?;
                Ok(Exp::Num(Number::from(bytes)))
            }
            _ => Err(Err::Reason("Expected second arg to be a list".to_string()))
        }
    }));
    data.insert("string".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        let args: Vec<String> = args.iter().map(|arg| match arg {
            Exp::Str(s) => format!("{}", s),
            exp => format!("{}", exp),
        }).collect();
        Ok(Exp::Str(args.join("")))
    }));
    data.insert("string->bytes".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let s = string(&args[0])?;
        let buf = s.as_bytes();
        Ok(Exp::List(buf.iter().map(|b| Exp::Num(Number::from(*b))).collect()))
    }));
    data.insert("bytes->string".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        match &args[0] {
            Exp::List(list) => {
                let buf = list_of_bytes(list)?;
                let s = String::from_utf8(buf).or(Err(Err::Reason("Could not convert to valid UTF-8 string".to_string())))?;
                Ok(Exp::Str(s))
            }
            _ => Err(Err::Reason("Expected arg to be a list".to_string()))
        }
    }));
    data.insert("bytes->number".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        match &args[0] {
            Exp::List(list) => {
                let bytes = list_of_bytes(list)?;
                ensure_length_eq!(bytes, 8);
                Ok(Exp::Num(Number::from(f64::from_be_bytes(bytes[0..8].try_into().unwrap()))))
            }
            _ => Err(Err::Reason("Expected arg to be a list".to_string()))
        }
    }));
    data.insert("number->bytes".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let f = float(&args[0])?;
        Ok(Exp::List(f.to_be_bytes().iter().map(|b| Exp::Num(Number::from(*b))).collect()))
    }));
    data.insert("regex-find".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 2);
        match (&args[0], &args[1]) {
            (Exp::Str(regex), Exp::Str(s)) => {
                let res = Regex::new(regex).find(s).map(|(a, b)| {
                    vec![Exp::Num(Number::from(a)), Exp::Num(Number::from(b))]
                }).unwrap_or(vec![]);
                Ok(Exp::List(res))
            }
            _ => Err(Err::Reason("Expected args to be a regex and a string".to_string()))
        }
    }));
    data.insert("lines".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let s = string(&args[0])?;
        let lines = s.lines().map(|line| Exp::Str(line.to_string())).collect();
        Ok(Exp::List(lines))
    }));
    data.insert("string->number".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let s = string(&args[0])?;
        let n = s.parse().or(Err(Err::Reason("Could not parse number".to_string())))?;
        Ok(Exp::Num(n))
    }));
    data.insert("type".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let exp = match args[0] {
            Exp::Str(_) => "string",
            Exp::Bool(_) => "boolean",
            Exp::Sym(_) => "symbol",
            Exp::Num(_) => "number",
            Exp::List(_) => "list",
            Exp::Primitive(_) => "function",
            Exp::Lambda(_) => "function",
        };
        Ok(Exp::Str(exp.to_string()))
    }));
    data.insert("number-type".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        match args[0] {
            Exp::Num(Number::Int(_)) => Ok(Exp::Str("int".to_string())),
            Exp::Num(Number::BigInt(_)) => Ok(Exp::Str("bigint".to_string())),
            Exp::Num(Number::Float(_)) => Ok(Exp::Str("float".to_string())),
            _ => Err(Err::Reason("Expected arg to be a number".to_string()))
        }
    }));
    data.insert("list".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        Ok(Exp::List(args.to_vec()))
    }));
    data.insert("parse".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let s = string(&args[0])?;
        let (_, exp) = parse(&s)?;
        Ok(exp)
    }));

    // Setup autocompletion
    *FORMS.lock() = data.keys().cloned().chain(BUILT_INS.map(String::from)).collect();

    Rc::new(RefCell::new(Env { data, outer: None }))
}

fn list_of_symbols(form: &Exp) -> Result<Vec<String>, Err> {
    match form {
        Exp::List(list) => {
            list.iter().map(|exp| {
                match exp {
                    Exp::Sym(sym) => Ok(sym.clone()),
                    _ => Err(Err::Reason("Expected symbols in the argument list".to_string()))
                }
            }).collect()
        }
        _ => Err(Err::Reason("Expected args form to be a list".to_string()))
    }
}

fn list_of_numbers(args: &[Exp]) -> Result<Vec<Number>, Err> {
    args.iter().map(number).collect()
}

fn list_of_floats(args: &[Exp]) -> Result<Vec<f64>, Err> {
    args.iter().map(float).collect()
}

fn list_of_bytes(args: &[Exp]) -> Result<Vec<u8>, Err> {
    args.iter().map(byte).collect()
}

fn string(exp: &Exp) -> Result<String, Err> {
    match exp {
        Exp::Str(s) => Ok(s.to_string()),
        _ => Err(Err::Reason("Expected a string".to_string())),
    }
}

fn number(exp: &Exp) -> Result<Number, Err> {
    match exp {
        Exp::Num(num) => Ok(num.clone()),
        _ => Err(Err::Reason("Expected a number".to_string())),
    }
}

fn float(exp: &Exp) -> Result<f64, Err> {
    match exp {
        Exp::Num(num) => Ok(num.into()),
        _ => Err(Err::Reason("Expected a float".to_string())),
    }
}

fn byte(exp: &Exp) -> Result<u8, Err> {
    number(exp)?.try_into()
}

// Eval

fn eval_quote_args(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(args[0].clone())
}

fn eval_atom_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::List(_) => Ok(Exp::Bool(false)),
        _            => Ok(Exp::Bool(true)),
    }
}

fn eval_eq_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let a = eval(&args[0], env)?;
    let b = eval(&args[1], env)?;
    Ok(Exp::Bool(a == b))
}

fn eval_car_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::List(list) => {
            ensure_length_gt!(list, 0);
            Ok(list[0].clone())
        },
        _ => Err(Err::Reason("Expected list form".to_string())),
    }
}

fn eval_cdr_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::List(list) => {
            ensure_length_gt!(list, 0);
            Ok(Exp::List(list[1..].to_vec()))
        },
        _ => Err(Err::Reason("Expected list form".to_string())),
    }
}

fn eval_cons_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match eval(&args[1], env)? {
        Exp::List(mut list) => {
            list.insert(0, eval(&args[0], env)?);
            Ok(Exp::List(list.to_vec()))
        },
        _ => Err(Err::Reason("Expected list form".to_string())),
    }
}

fn eval_cond_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    for arg in args {
        match arg {
            Exp::List(list) => {
                ensure_length_eq!(list, 2);
                let pred = eval(&list[0], env)?;
                let exp = eval(&list[1], env)?;
                match pred {
                    Exp::Bool(b) if b => return Ok(exp),
                    _ => continue,
                }
            },
            _ => return Err(Err::Reason("Expected lists of predicate and expression".to_string())),
        }
    }
    Ok(Exp::List(Vec::new()))
}

fn eval_label_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match &args[0] {
        Exp::Sym(name) => {
            let exp = eval(&args[1], env)?;
            env.borrow_mut().data.insert(name.clone(), exp);
            Ok(Exp::Sym(name.clone()))
        }
        Exp::List(params) => {
            // (label (add x y) (+ x y)) => (label add (lambda (x y) (+ x y)))
            ensure_length_gt!(params, 0);
            let name = params[0].clone();
            let params = Exp::List(params[1..].to_vec());
            let body = args[1].clone();
            let lambda_args = vec![Exp::Sym("lambda".to_string()), params, body];
            let label_args = vec![name, Exp::List(lambda_args)];
            eval_label_args(&label_args, env)
        }
        _ => Err(Err::Reason("Expected first argument to be a symbol or a list".to_string()))
    }
}

fn eval_lambda_args(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    Ok(Exp::Lambda(Lambda {
        params: Rc::new(args[0].clone()),
        body: Rc::new(args[1].clone()),
    }))
}

fn eval_defun_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    // (defun add (x y) (+ x y)) => (label add (lambda (x y) (+ x y)))
    ensure_length_eq!(args, 3);
    let name = args[0].clone();
    let params = args[1].clone();
    let body = args[2].clone();
    let lambda_args = vec![Exp::Sym("lambda".to_string()), params, body];
    let label_args = vec![name, Exp::List(lambda_args)];
    eval_label_args(&label_args, env)
}

fn eval_apply_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_gt!(args, 1);
    let mut args = args.to_vec();
    match eval(&args.pop().unwrap(), env) {
        Ok(Exp::List(rest)) => args.extend(rest),
        _ => return Err(Err::Reason("Expected last argument to be a list".to_string())),
    }
    eval(&Exp::List(args.to_vec()), env)
}

fn eval_eval_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let exp = eval(&args[0], env)?;
    eval(&exp, env)
}

fn eval_progn_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    let mut res = Ok(Exp::List(vec![]));
    for arg in args {
        res = Ok(eval(arg, env)?);
    }
    res
}

fn eval_load_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let path = string(&args[0])?;
    let mut code = fs::read_to_string(&path).or(Err(Err::Reason("Could not read file".to_string())))?;
    loop {
        let (rest, exp) = parse(&code)?;
        eval(&exp, env)?;
        if rest.is_empty() {
            break;
        }
        code = rest;
    }
    Ok(Exp::Bool(true))
}

const BUILT_INS: [&str; 22] = [
    "quote", "atom", "eq", "car", "cdr", "cons", "cond", "label", "lambda", "define", "def",
    "function", "fun", "fn", "defun", "defn", "apply", "eval", "progn", "begin", "do", "load"
];

fn eval_built_in_form(exp: &Exp, args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Option<Result<Exp, Err>> {
    match exp {
        Exp::Sym(s) => {
            match s.as_ref() {
                // Seven Primitive Operators
                "quote"                      => Some(eval_quote_args(args)),
                "atom"                       => Some(eval_atom_args(args, env)),
                "eq"                         => Some(eval_eq_args(args, env)),
                "car"                        => Some(eval_car_args(args, env)),
                "cdr"                        => Some(eval_cdr_args(args, env)),
                "cons"                       => Some(eval_cons_args(args, env)),
                "cond"                       => Some(eval_cond_args(args, env)),

                // Two Special Forms
                "label" | "define" | "def"   => Some(eval_label_args(args, env)),
                "lambda" | "function" | "fn" => Some(eval_lambda_args(args)),

                "defun" | "defn"             => Some(eval_defun_args(args, env)),
                "apply"                      => Some(eval_apply_args(args, env)),
                "eval"                       => Some(eval_eval_args(args, env)),
                "progn" | "begin" | "do"     => Some(eval_progn_args(args, env)),
                "load"                       => Some(eval_load_args(args, env)),
                _                            => None,
            }
        },
        _ => None,
    }
}

fn env_get(key: &str, env: &Rc<RefCell<Env>>) -> Result<Exp, Err> {
    let env = env.borrow_mut();
    match env.data.get(key) {
        Some(exp) => Ok(exp.clone()),
        None => {
            match &env.outer {
                Some(outer_env) => env_get(key, outer_env.borrow()),
                None => Err(Err::Reason(format!("Unexpected symbol '{}'", key))),
            }
        }
    }
}

fn lambda_env(params: Rc<Exp>, args: &[Exp], outer: &mut Rc<RefCell<Env>>) -> Result<Rc<RefCell<Env>>, Err> {
    let ks = list_of_symbols(&params)?;
    if ks.len() != args.len() {
        let plural = if ks.len() == 1 { "" } else { "s" };
        return Err(Err::Reason(format!("Expected {} argument{}, got {}", ks.len(), plural, args.len())));
    }
    let vs = eval_args(args, outer)?;
    let mut data: BTreeMap<String, Exp> = BTreeMap::new();
    for (k, v) in ks.iter().zip(vs.iter()) {
        data.insert(k.clone(), v.clone());
    }
    Ok(Rc::new(RefCell::new(Env { data, outer: Some(Rc::new(RefCell::new(outer.borrow_mut().clone()))) })))
}

fn eval_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Vec<Exp>, Err> {
    args.iter().map(|x| eval(x, env)).collect()
}

fn eval(exp: &Exp, env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    match exp {
        Exp::Sym(key) => env_get(&key, &env),
        Exp::Bool(_) => Ok(exp.clone()),
        Exp::Num(_) => Ok(exp.clone()),
        Exp::Str(_) => Ok(exp.clone()),
        Exp::List(list) => {
            ensure_length_gt!(list, 0);
            let first_form = &list[0];
            let args = &list[1..];
            match eval_built_in_form(first_form, args, env) {
                Some(res) => res,
                None => {
                    let first_eval = eval(first_form, env)?;
                    match first_eval {
                        Exp::Primitive(f) => {
                            f(&eval_args(args, env)?)
                        },
                        Exp::Lambda(f) => {
                            let mut env = lambda_env(f.params, args, env)?;
                            eval(&f.body, &mut env)
                        },
                        _ => Err(Err::Reason("First form must be a function".to_string())),
                    }
                }
            }
        },
        Exp::Primitive(_) => Err(Err::Reason("Unexpected form".to_string())),
        Exp::Lambda(_) => Err(Err::Reason("Unexpected form".to_string())),
    }
}

// REPL

fn parse_eval(exp: &str, env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    let (_, exp) = parse(exp)?;
    let exp = eval(&exp, env)?;
    Ok(exp)
}

fn strip_comments(s: &str) -> String {
    s.split('#').next().unwrap().into()
}

fn repl(env: &mut Rc<RefCell<Env>>) -> Result<(), ExitCode> {
    let csi_color = Style::color("Cyan");
    let csi_error = Style::color("LightRed");
    let csi_reset = Style::reset();
    let prompt_string = format!("{}>{} ", csi_color, csi_reset);

    println!("MOROS Lisp v0.4.0\n");

    let mut prompt = Prompt::new();
    let history_file = "~/.lisp-history";
    prompt.history.load(history_file);
    prompt.completion.set(&lisp_completer);

    while let Some(line) = prompt.input(&prompt_string) {
        if line == "(quit)" {
            break;
        }
        if line.is_empty() {
            println!();
            continue;
        }
        match parse_eval(&line, env) {
            Ok(res) => {
                println!("{}\n", res);
            }
            Err(e) => match e {
                Err::Reason(msg) => println!("{}Error:{} {}\n", csi_error, csi_reset, msg),
            },
        }
        prompt.history.add(&line);
        prompt.history.save(history_file);
    }
    Ok(())
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let line_color = Style::color("Yellow");
    let error_color = Style::color("LightRed");
    let reset = Style::reset();

    let env = &mut default_env();

    // Store args in env
    let key = Exp::Sym("args".to_string());
    let list = Exp::List(if args.len() < 2 {
        vec![]
    } else {
        args[2..].iter().map(|arg| Exp::Str(arg.to_string())).collect()
    });
    let quote = Exp::List(vec![Exp::Sym("quote".to_string()), list]);
    if eval_label_args(&[key, quote], env).is_err() {
        error!("Could not parse args");
        return Err(ExitCode::Failure);
    }

    if args.len() < 2 {
        repl(env)
    } else {
        let pathname = args[1];
        if let Ok(code) = api::fs::read_to_string(pathname) {
            let mut block = String::new();
            let mut opened = 0;
            let mut closed = 0;
            for (i, line) in code.split('\n').enumerate() {
                let line = strip_comments(line);
                if !line.is_empty() {
                    opened += line.matches('(').count();
                    closed += line.matches(')').count();
                    block.push_str(&line);
                    if closed >= opened {
                        if let Err(e) = parse_eval(&block, env) {
                            match e {
                                Err::Reason(msg) => {
                                    eprintln!("{}Error:{} {}", error_color, reset, msg);
                                    eprintln!();
                                    eprintln!("  {}{}:{} {}", line_color, i, reset, line);
                                    return Err(ExitCode::Failure);
                                }
                            }
                        }
                        block.clear();
                        opened = 0;
                        closed = 0;
                    }
                }
            }
            Ok(())
        } else {
            error!("File not found '{}'", pathname);
            Err(ExitCode::Failure)
        }
    }
}

#[test_case]
fn test_lisp() {
    let env = &mut default_env();

    macro_rules! eval {
        ($e:expr) => {
            format!("{}", parse_eval($e, env).unwrap())
        };
    }

    // quote
    assert_eq!(eval!("(quote (1 2 3))"), "(1 2 3)");
    assert_eq!(eval!("'(1 2 3)"), "(1 2 3)");
    assert_eq!(eval!("(quote 1)"), "1");
    assert_eq!(eval!("'1"), "1");
    assert_eq!(eval!("(quote a)"), "a");
    assert_eq!(eval!("'a"), "a");
    assert_eq!(eval!("(quote '(a b c))"), "(quote (a b c))");

    // atom
    assert_eq!(eval!("(atom (quote a))"), "true");
    assert_eq!(eval!("(atom (quote (1 2 3)))"), "false");
    assert_eq!(eval!("(atom 1)"), "true");

    // eq
    assert_eq!(eval!("(eq (quote a) (quote a))"), "true");
    assert_eq!(eval!("(eq (quote a) (quote b))"), "false");
    assert_eq!(eval!("(eq (quote a) (quote ()))"), "false");
    assert_eq!(eval!("(eq (quote ()) (quote ()))"), "true");
    assert_eq!(eval!("(eq \"a\" \"a\")"), "true");
    assert_eq!(eval!("(eq \"a\" \"b\")"), "false");
    assert_eq!(eval!("(eq \"a\" 'b)"), "false");
    assert_eq!(eval!("(eq 1 1)"), "true");
    assert_eq!(eval!("(eq 1 2)"), "false");
    assert_eq!(eval!("(eq 1 1.0)"), "false");
    assert_eq!(eval!("(eq 1.0 1.0)"), "true");

    // car
    assert_eq!(eval!("(car (quote (1)))"), "1");
    assert_eq!(eval!("(car (quote (1 2 3)))"), "1");

    // cdr
    assert_eq!(eval!("(cdr (quote (1)))"), "()");
    assert_eq!(eval!("(cdr (quote (1 2 3)))"), "(2 3)");

    // cons
    assert_eq!(eval!("(cons (quote 1) (quote (2 3)))"), "(1 2 3)");
    assert_eq!(eval!("(cons (quote 1) (cons (quote 2) (cons (quote 3) (quote ()))))"), "(1 2 3)");

    // cond
    assert_eq!(eval!("(cond ((< 2 4) 1))"), "1");
    assert_eq!(eval!("(cond ((> 2 4) 1))"), "()");
    assert_eq!(eval!("(cond ((< 2 4) 1) (true 2))"), "1");
    assert_eq!(eval!("(cond ((> 2 4) 1) (true 2))"), "2");

    // label
    eval!("(label a 2)");
    assert_eq!(eval!("(+ a 1)"), "3");
    //eval!("(label fn lambda)");
    //assert_eq!(eval!("((fn (a) (+ 1 a)) 2)"), "3");
    eval!("(label add-one (lambda (b) (+ b 1)))");
    assert_eq!(eval!("(add-one 2)"), "3");
    eval!("(label fib (lambda (n) (cond ((< n 2) n) (true (+ (fib (- n 1)) (fib (- n 2)))))))");
    assert_eq!(eval!("(fib 6)"), "8");

    // lambda
    assert_eq!(eval!("((lambda (a) (+ 1 a)) 2)"), "3");
    assert_eq!(eval!("((lambda (a) (* a a)) 2)"), "4");
    assert_eq!(eval!("((lambda (x) (cons x '(b c))) 'a)"), "(a b c)");

    // defun
    eval!("(defun add (a b) (+ a b))");
    assert_eq!(eval!("(add 1 2)"), "3");

    // addition
    assert_eq!(eval!("(+)"), "0");
    assert_eq!(eval!("(+ 2)"), "2");
    assert_eq!(eval!("(+ 2 2)"), "4");
    assert_eq!(eval!("(+ 2 3 4)"), "9");
    assert_eq!(eval!("(+ 2 (+ 3 4))"), "9");

    // subtraction
    assert_eq!(eval!("(- 2)"), "-2");
    assert_eq!(eval!("(- 2 1)"), "1");
    assert_eq!(eval!("(- 1 2)"), "-1");
    assert_eq!(eval!("(- 2 -1)"), "3");
    assert_eq!(eval!("(- 8 4 2)"), "2");

    // multiplication
    assert_eq!(eval!("(*)"), "1");
    assert_eq!(eval!("(* 2)"), "2");
    assert_eq!(eval!("(* 2 2)"), "4");
    assert_eq!(eval!("(* 2 3 4)"), "24");
    assert_eq!(eval!("(* 2 (* 3 4))"), "24");

    // division
    assert_eq!(eval!("(/ 4)"), "0");
    assert_eq!(eval!("(/ 4.0)"), "0.25");
    assert_eq!(eval!("(/ 4 2)"), "2");
    assert_eq!(eval!("(/ 1 2)"), "0");
    assert_eq!(eval!("(/ 1 2.0)"), "0.5");
    assert_eq!(eval!("(/ 8 4 2)"), "1");

    // exponential
    assert_eq!(eval!("(^ 2 4)"), "16");
    assert_eq!(eval!("(^ 2 4 2)"), "256"); // Left to right

    // modulo
    assert_eq!(eval!("(% 3 2)"), "1");

    // comparisons
    assert_eq!(eval!("(< 6 4)"), "false");
    assert_eq!(eval!("(> 6 4 3 1)"), "true");
    assert_eq!(eval!("(= 6 4)"), "false");
    assert_eq!(eval!("(= 6 6)"), "true");
    assert_eq!(eval!("(= (+ 0.15 0.15) (+ 0.1 0.2))"), "true");

    // number
    assert_eq!(eval!("(bytes->number (number->bytes 42.0))"), "42.0");

    // string
    assert_eq!(eval!("(parse \"9.75\")"), "9.75");
    assert_eq!(eval!("(string \"a\" \"b\" \"c\")"), "\"abc\"");
    assert_eq!(eval!("(string \"a\" \"\")"), "\"a\"");
    assert_eq!(eval!("(string \"foo \" 3)"), "\"foo 3\"");
    assert_eq!(eval!("(eq \"foo\" \"foo\")"), "true");
    assert_eq!(eval!("(eq \"foo\" \"bar\")"), "false");
    assert_eq!(eval!("(lines \"a\nb\nc\")"), "(\"a\" \"b\" \"c\")");

    // apply
    assert_eq!(eval!("(apply + '(1 2 3))"), "6");
    assert_eq!(eval!("(apply + 1 '(2 3))"), "6");
    assert_eq!(eval!("(apply + 1 2 '(3))"), "6");
    assert_eq!(eval!("(apply + 1 2 3 '())"), "6");

    // trigo
    assert_eq!(eval!("(acos (cos pi))"), PI.to_string());
    assert_eq!(eval!("(acos 0)"), (PI / 2.0).to_string());
    assert_eq!(eval!("(asin 1)"), (PI / 2.0).to_string());
    assert_eq!(eval!("(atan 0)"), "0.0");
    assert_eq!(eval!("(cos pi)"), "-1.0");
    assert_eq!(eval!("(sin (/ pi 2))"), "1.0");
    assert_eq!(eval!("(tan 0)"), "0.0");

    // list
    assert_eq!(eval!("(list)"), "()");
    assert_eq!(eval!("(list 1)"), "(1)");
    assert_eq!(eval!("(list 1 2)"), "(1 2)");
    assert_eq!(eval!("(list 1 2 (+ 1 2))"), "(1 2 3)");

    // bigint
    assert_eq!(eval!("9223372036854775807"),          "9223372036854775807");   // -> int
    assert_eq!(eval!("9223372036854775808"),          "9223372036854775808");   // -> bigint
    assert_eq!(eval!("(+ 9223372036854775807 0)"),    "9223372036854775807");   // -> int
    assert_eq!(eval!("(- 9223372036854775808 1)"),    "9223372036854775807");   // -> bigint
    assert_eq!(eval!("(+ 9223372036854775807 1)"),    "9223372036854775808");   // -> bigint
    assert_eq!(eval!("(+ 9223372036854775807 1.0)"),  "9223372036854776000.0"); // -> float
    assert_eq!(eval!("(+ 9223372036854775807 10)"),   "9223372036854775817");   // -> bigint
    assert_eq!(eval!("(* 9223372036854775807 10)"),  "92233720368547758070");   // -> bigint

    assert_eq!(eval!("(^ 2 16)"),                                      "65536");   // -> int
    assert_eq!(eval!("(^ 2 128)"),   "340282366920938463463374607431768211456");   // -> bigint
    assert_eq!(eval!("(^ 2.0 128)"), "340282366920938500000000000000000000000.0"); // -> float

    assert_eq!(eval!("(number-type 9223372036854775807)"),   "\"int\"");
    assert_eq!(eval!("(number-type 9223372036854775808)"),   "\"bigint\"");
    assert_eq!(eval!("(number-type 9223372036854776000.0)"), "\"float\"");
}
