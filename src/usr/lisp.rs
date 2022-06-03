use crate::{api, usr};
use crate::api::fs;
use crate::api::console::Style;
use crate::api::prompt::Prompt;
use alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::vec;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use core::fmt;
use float_cmp::approx_eq;

use nom::IResult;
use nom::combinator::value;
use nom::bytes::complete::escaped_transform;
use nom::branch::alt;
use nom::character::complete::char;
use nom::character::complete::multispace0;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while1;
use nom::bytes::complete::is_not;
use nom::sequence::preceded;
use nom::multi::many0;
use nom::number::complete::double;
use nom::sequence::delimited;

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

#[derive(Clone)]
enum Exp {
    Lambda(Lambda),
    Func(fn(&[Exp]) -> Result<Exp, Err>),
    List(Vec<Exp>),
    Bool(bool),
    Num(f64),
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
        let str = match self {
            Exp::Lambda(_)  => "Lambda {}".to_string(),
            Exp::Func(_)    => "Function {}".to_string(),
            Exp::Bool(a)    => a.to_string(),
            Exp::Num(n)     => n.to_string(),
            Exp::Sym(s)     => s.clone(),
            Exp::Str(s)     => format!("\"{}\"", s.replace('"', "\\\"")),
            Exp::List(list) => {
                let xs: Vec<String> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(" "))
            },
        };
        write!(f, "{}", str)
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
struct Env<'a> {
    data: BTreeMap<String, Exp>,
    outer: Option<&'a Env<'a>>,
}

const COMPLETER_FORMS: [&str; 21] = [
    "atom",
    "bytes",
    "car",
    "cdr",
    "cond",
    "cons",
    "defun",
    "eq",
    "label",
    "lambda",
    "lines",
    "load",
    "mapcar",
    "parse",
    "print",
    "progn",
    "quote",
    "read",
    "read-bytes",
    "str",
    "type",
];

fn lisp_completer(line: &str) -> Vec<String> {
    let mut entries = Vec::new();
    if let Some(last_word) = line.split_whitespace().next_back() {
        if let Some(f) = last_word.strip_prefix('(') {
            for form in COMPLETER_FORMS {
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
    let escaped = escaped_transform(is_not("\\\""), '\\', alt((
        value("\\", tag("\\")),
        value("\"", tag("\"")),
        value("\n", tag("n")),
    )));
    let (input, s) = delimited(char('"'), escaped, char('"'))(input)?;
    Ok((input, Exp::Str(s)))
}

fn parse_sym(input: &str) -> IResult<&str, Exp> {
    let (input, sym) = take_while1(is_symbol_letter)(input)?;
    Ok((input, Exp::Sym(sym.to_string())))
}

fn parse_num(input: &str) -> IResult<&str, Exp> {
    let (input, num) = double(input)?;
    Ok((input, Exp::Num(num)))
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
            fn func(prev: &f64, xs: &[f64]) -> bool {
                match xs.first() {
                    Some(x) => $check_fn(*prev, *x) && func(x, &xs[1..]),
                    None => true,
                }
            }
            Ok(Exp::Bool(func(first, rest)))
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

fn default_env<'a>() -> Env<'a> {
    let mut data: BTreeMap<String, Exp> = BTreeMap::new();
    data.insert("=".to_string(), Exp::Func(ensure_tonicity!(|a, b| approx_eq!(f64, a, b))));
    data.insert(">".to_string(), Exp::Func(ensure_tonicity!(|a, b| !approx_eq!(f64, a, b) && a > b)));
    data.insert(">=".to_string(), Exp::Func(ensure_tonicity!(|a, b| approx_eq!(f64, a, b) || a > b)));
    data.insert("<".to_string(), Exp::Func(ensure_tonicity!(|a, b| !approx_eq!(f64, a, b) && a < b)));
    data.insert("<=".to_string(), Exp::Func(ensure_tonicity!(|a, b| approx_eq!(f64, a, b) || a < b)));
    data.insert("*".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let res = list_of_floats(args)?.iter().fold(1.0, |acc, a| acc * a);
        Ok(Exp::Num(res))
    }));
    data.insert("+".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let res = list_of_floats(args)?.iter().fold(0.0, |acc, a| acc + a);
        Ok(Exp::Num(res))
    }));
    data.insert("-".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let args = list_of_floats(args)?;
        ensure_length_gt!(args, 0);
        let car = args[0];
        let cdr = args[1..].iter().fold(0.0, |acc, a| acc + a);
        Ok(Exp::Num(car - cdr))
    }));
    data.insert("/".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let args = list_of_floats(args)?;
        ensure_length_gt!(args, 0);
        let car = args[0];
        let res = args[1..].iter().fold(car, |acc, a| acc / a);
        Ok(Exp::Num(res))
    }));
    data.insert("%".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let args = list_of_floats(args)?;
        ensure_length_gt!(args, 0);
        let car = args[0];
        let res = args[1..].iter().fold(car, |acc, a| libm::fmod(acc, *a));
        Ok(Exp::Num(res))
    }));
    data.insert("^".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let args = list_of_floats(args)?;
        ensure_length_gt!(args, 0);
        let car = args[0];
        let res = args[1..].iter().fold(car, |acc, a| libm::pow(acc, *a));
        Ok(Exp::Num(res))
    }));
    data.insert("print".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        match args[0].clone() {
            Exp::Str(s) => {
                print!("{}", s);
                Ok(Exp::Str(s))
            }
            exp => {
                print!("{}", exp);
                Ok(exp)
            }
        }
    }));
    data.insert("read".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let path = string(&args[0])?;
        let contents = fs::read_to_string(&path).or(Err(Err::Reason("Could not read file".to_string())))?;
        Ok(Exp::Str(contents))
    }));
    data.insert("read-bytes".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 2);
        let path = string(&args[0])?;
        let len = float(&args[1])?;
        let mut buf = vec![0; len as usize];
        let bytes = fs::read(&path, &mut buf).or(Err(Err::Reason("Could not read file".to_string())))?;
        buf.resize(bytes, 0);
        Ok(Exp::List(buf.iter().map(|b| Exp::Num(*b as f64)).collect()))
    }));
    data.insert("bytes".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let s = string(&args[0])?;
        let buf = s.as_bytes();
        Ok(Exp::List(buf.iter().map(|b| Exp::Num(*b as f64)).collect()))
    }));
    data.insert("str".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        match &args[0] {
            Exp::List(list) => {
                let buf = list_of_floats(list)?.iter().map(|b| *b as u8).collect();
                let s = String::from_utf8(buf).or(Err(Err::Reason("Could not convert to valid UTF-8 string".to_string())))?;
                Ok(Exp::Str(s))
            }
            _ => Err(Err::Reason("Expected arg to be a list".to_string()))
        }
    }));
    data.insert("lines".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let s = string(&args[0])?;
        let lines = s.lines().map(|line| Exp::Str(line.to_string())).collect();
        Ok(Exp::List(lines))
    }));
    data.insert("parse".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let s = string(&args[0])?;
        let n = s.parse().or(Err(Err::Reason("Could not parse number".to_string())))?;
        Ok(Exp::Num(n))
    }));
    data.insert("type".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let exp = match args[0] {
            Exp::Str(_) => "string",
            Exp::Bool(_) => "boolean",
            Exp::Sym(_) => "symbol",
            Exp::Num(_) => "number",
            Exp::List(_) => "list",
            Exp::Func(_) => "function",
            Exp::Lambda(_) => "lambda",
        };
        Ok(Exp::Str(exp.to_string()))
    }));
    Env { data, outer: None }
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

fn list_of_floats(args: &[Exp]) -> Result<Vec<f64>, Err> {
    args.iter().map(float).collect()
}

fn float(exp: &Exp) -> Result<f64, Err> {
    match exp {
        Exp::Num(num) => Ok(*num),
        _ => Err(Err::Reason("Expected a number".to_string())),
    }
}

fn string(exp: &Exp) -> Result<String, Err> {
    match exp {
        Exp::Str(s) => Ok(s.to_string()),
        _ => Err(Err::Reason("Expected a string".to_string())),
    }
}

// Eval

fn eval_quote_args(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(args[0].clone())
}

fn eval_atom_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::List(_) => Ok(Exp::Bool(false)),
        _            => Ok(Exp::Bool(true)),
    }
}

fn eval_eq_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let a = eval(&args[0], env)?;
    let b = eval(&args[1], env)?;
    Ok(Exp::Bool(a == b))
}

fn eval_car_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::List(list) => {
            ensure_length_gt!(list, 0);
            Ok(list[0].clone())
        },
        _ => Err(Err::Reason("Expected list form".to_string())),
    }
}

fn eval_cdr_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::List(list) => {
            ensure_length_gt!(list, 0);
            Ok(Exp::List(list[1..].to_vec()))
        },
        _ => Err(Err::Reason("Expected list form".to_string())),
    }
}

fn eval_cons_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match eval(&args[1], env)? {
        Exp::List(mut list) => {
            list.insert(0, eval(&args[0], env)?);
            Ok(Exp::List(list.to_vec()))
        },
        _ => Err(Err::Reason("Expected list form".to_string())),
    }
}

fn eval_cond_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
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

fn eval_label_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match &args[0] {
        Exp::Sym(key) => {
            let exp = eval(&args[1], env)?;
            env.data.insert(key.clone(), exp);
            Ok(Exp::Sym(key.clone()))
        }
        _ => Err(Err::Reason("Expected first argument to be a symbol".to_string()))
    }
}

fn eval_lambda_args(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    Ok(Exp::Lambda(Lambda {
        params: Rc::new(args[0].clone()),
        body: Rc::new(args[1].clone()),
    }))
}

fn eval_defun_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    ensure_length_eq!(args, 3);
    let name = args[0].clone();
    let params = args[1].clone();
    let exp = args[2].clone();
    let lambda_args = vec![Exp::Sym("lambda".to_string()), params, exp];
    let label_args = vec![name, Exp::List(lambda_args)];
    eval_label_args(&label_args, env)
}

fn eval_mapcar_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match eval(&args[1], env) {
        Ok(Exp::List(list)) => {
            Ok(Exp::List(list.iter().map(|exp| {
                eval(&Exp::List(vec!(args[0].clone(), exp.clone())), env)
            }).collect::<Result<Vec<Exp>, Err>>()?))
        }
        _ => Err(Err::Reason("Expected second argument to be a list".to_string())),
    }
}

fn eval_progn_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    let mut res = Ok(Exp::List(vec![]));
    for arg in args {
        res = Ok(eval(arg, env)?);
    }
    res
}

fn eval_load_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
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

fn eval_built_in_form(exp: &Exp, args: &[Exp], env: &mut Env) -> Option<Result<Exp, Err>> {
    match exp {
        Exp::Sym(s) => {
            match s.as_ref() {
                // Seven Primitive Operators
                "quote"          => Some(eval_quote_args(args)),
                "atom"           => Some(eval_atom_args(args, env)),
                "eq"             => Some(eval_eq_args(args, env)),
                "car"            => Some(eval_car_args(args, env)),
                "cdr"            => Some(eval_cdr_args(args, env)),
                "cons"           => Some(eval_cons_args(args, env)),
                "cond"           => Some(eval_cond_args(args, env)),

                // Two Special Forms
                "label" | "def"  => Some(eval_label_args(args, env)),
                "lambda" | "fn"  => Some(eval_lambda_args(args)),

                "defun" | "defn" => Some(eval_defun_args(args, env)),
                "mapcar" | "map" => Some(eval_mapcar_args(args, env)),
                "progn" | "do"   => Some(eval_progn_args(args, env)),
                "load"           => Some(eval_load_args(args, env)),
                _                => None,
            }
        },
        _ => None,
    }
}

fn env_get(key: &str, env: &Env) -> Result<Exp, Err> {
    match env.data.get(key) {
        Some(exp) => Ok(exp.clone()),
        None => {
            match &env.outer {
                Some(outer_env) => env_get(key, outer_env),
                None => Err(Err::Reason(format!("Unexpected symbol '{}'", key))),
            }
        }
    }
}

fn env_for_lambda<'a>(params: Rc<Exp>, args: &[Exp], outer: &'a mut Env) -> Result<Env<'a>, Err> {
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
    Ok(Env { data, outer: Some(outer) })
}

fn eval_args(args: &[Exp], env: &mut Env) -> Result<Vec<Exp>, Err> {
    args.iter().map(|x| eval(x, env)).collect()
}

fn eval(exp: &Exp, env: &mut Env) -> Result<Exp, Err> {
    match exp {
        Exp::Sym(key) => env_get(key, env),
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
                        Exp::Func(func) => {
                            func(&eval_args(args, env)?)
                        },
                        Exp::Lambda(lambda) => {
                            let env = &mut env_for_lambda(lambda.params, args, env)?;
                            eval(&lambda.body, env)
                        },
                        _ => Err(Err::Reason("First form must be a function".to_string())),
                    }
                }
            }
        },
        Exp::Func(_) => Err(Err::Reason("Unexpected form".to_string())),
        Exp::Lambda(_) => Err(Err::Reason("Unexpected form".to_string())),
    }
}

// REPL

fn parse_eval(exp: &str, env: &mut Env) -> Result<Exp, Err> {
    let (_, exp) = parse(exp)?;
    let exp = eval(&exp, env)?;
    Ok(exp)
}

fn strip_comments(s: &str) -> String {
    s.split('#').next().unwrap().into()
}

fn repl(env: &mut Env) -> usr::shell::ExitCode {
    let csi_color = Style::color("Cyan");
    let csi_error = Style::color("LightRed");
    let csi_reset = Style::reset();
    let prompt_string = format!("{}>{} ", csi_color, csi_reset);

    println!("MOROS Lisp v0.2.0\n");

    let mut prompt = Prompt::new();
    let history_file = "~/.lisp-history";
    prompt.history.load(history_file);
    prompt.completion.set(&lisp_completer);

    while let Some(line) = prompt.input(&prompt_string) {
        if line == "(exit)" || line == "(quit)" {
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
    usr::shell::ExitCode::CommandSuccessful
}

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
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
        return usr::shell::ExitCode::CommandError;
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
                                    return usr::shell::ExitCode::CommandError;
                                }
                            }
                        }
                        block.clear();
                        opened = 0;
                        closed = 0;
                    }
                }
            }
            usr::shell::ExitCode::CommandSuccessful
        } else {
            error!("File not found '{}'", pathname);
            usr::shell::ExitCode::CommandError
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
    assert_eq!(eval!("(eq 1 1.0)"), "true");

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
    assert_eq!(eval!("(+ 2 2)"), "4");
    assert_eq!(eval!("(+ 2 3 4)"), "9");
    assert_eq!(eval!("(+ 2 (+ 3 4))"), "9");

    // subtraction
    assert_eq!(eval!("(- 8 4 2)"), "2");
    assert_eq!(eval!("(- 2 1)"), "1");
    assert_eq!(eval!("(- 1 2)"), "-1");
    assert_eq!(eval!("(- 2 -1)"), "3");

    // multiplication
    assert_eq!(eval!("(* 2 2)"), "4");
    assert_eq!(eval!("(* 2 3 4)"), "24");
    assert_eq!(eval!("(* 2 (* 3 4))"), "24");

    // division
    assert_eq!(eval!("(/ 4 2)"), "2");
    assert_eq!(eval!("(/ 1 2)"), "0.5");
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

    // string
    assert_eq!(eval!("(eq \"Hello, World!\" \"foo\")"), "false");
    assert_eq!(eval!("(lines \"a\nb\nc\")"), "(\"a\" \"b\" \"c\")");
    assert_eq!(eval!("(parse \"9.75\")"), "9.75");

    // map
    eval!("(defun inc (a) (+ a 1))");
    assert_eq!(eval!("(map inc '(1 2))"), "(2 3)");
    assert_eq!(eval!("(map parse '(\"1\" \"2\" \"3\"))"), "(1 2 3)");
    assert_eq!(eval!("(map (fn (n) (* n 2)) '(1 2 3))"), "(2 4 6)");

    eval!("(defn apply2 (f arg1 arg2) (f arg1 arg2))");
    assert_eq!(eval!("(apply2 + 1 2)"), "3");
}
