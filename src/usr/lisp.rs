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
// MOROS Lisp is also inspired by Racket and Clojure

// Types

#[derive(Clone)]
enum Exp {
    Bool(bool),
    Func(fn(&[Exp]) -> Result<Exp, Err>),
    Lambda(Lambda),
    List(Vec<Exp>),
    Num(f64),
    Str(String),
    Sym(String),
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Exp::Str(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            Exp::Bool(a) => a.to_string(),
            Exp::Sym(s) => s.clone(),
            Exp::Num(n) => n.to_string(),
            Exp::List(list) => {
                let xs: Vec<String> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(" "))
            },
            Exp::Func(_) => "Function {}".to_string(),
            Exp::Lambda(_) => "Lambda {}".to_string(),
        };

        write!(f, "{}", str)
    }
}

#[derive(Clone)]
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
    if let Ok((input, exp)) = parse_exp(input) {
        Ok((input.to_string(), exp))
    } else {
        Err(Err::Reason("Could not parse input".to_string()))
    }
}

// Env

macro_rules! ensure_tonicity {
    ($check_fn:expr) => {
        |args: &[Exp]| -> Result<Exp, Err> {
            let floats = list_of_floats(args)?;
            let first = floats.first().ok_or(Err::Reason("Expected at least one number".to_string()))?;
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

macro_rules! ensure_length {
    ($list:expr, $count:expr) => {
        if $list.len() != $count {
            let plural = if $count != 1 { "s" } else { "" };
            return Err(Err::Reason(format!("Expected {} expression{}", $count, plural)))
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
        let car = *args.first().ok_or(Err::Reason("Expected at least one number".to_string()))?;
        let cdr = args[1..].iter().fold(0.0, |acc, a| acc + a);
        Ok(Exp::Num(car - cdr))
    }));
    data.insert("/".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let args = list_of_floats(args)?;
        let first = *args.first().ok_or(Err::Reason("Expected at least one number".to_string()))?;
        let res = args[1..].iter().fold(first, |acc, a| acc / a);
        Ok(Exp::Num(res))
    }));
    data.insert("%".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let args = list_of_floats(args)?;
        let first = *args.first().ok_or(Err::Reason("Expected at least one number".to_string()))?;
        let res = args[1..].iter().fold(first, |acc, a| libm::fmod(acc, *a));
        Ok(Exp::Num(res))
    }));
    data.insert("^".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let args = list_of_floats(args)?;
        let first = *args.first().ok_or(Err::Reason("Expected at least one number".to_string()))?;
        let res = args[1..].iter().fold(first, |acc, a| libm::pow(acc, *a));
        Ok(Exp::Num(res))
    }));

    data.insert("print".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        match first(args)? {
            Exp::Str(s) => {
                println!("{}", s);
                Ok(Exp::Str(s))
            },
            res => {
                println!("{}", res);
                Ok(res)
            }
        }
    }));
    data.insert("read".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let arg = first(args)?;
        let path = string(&arg)?;
        let contents = fs::read_to_string(&path).or(Err(Err::Reason("Could not read file".to_string())))?;
        Ok(Exp::Str(contents))
    }));
    data.insert("lines".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let arg = first(args)?;
        let s = string(&arg)?;
        let lines = s.lines().map(|line| Exp::Str(line.to_string())).collect();
        Ok(Exp::List(lines))
    }));
    data.insert("parse".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let arg = first(args)?;
        let s = string(&arg)?;
        let n = s.parse().or(Err(Err::Reason("Could not parse number".to_string())))?;
        Ok(Exp::Num(n))
    }));
    data.insert("type".to_string(), Exp::Func(|args: &[Exp]| -> Result<Exp, Err> {
        let arg = first(args)?;
        let res = match arg {
            Exp::Str(_) => "string",
            Exp::Bool(_) => "boolean",
            Exp::Sym(_) => "symbol",
            Exp::Num(_) => "number",
            Exp::List(_) => "list",
            Exp::Func(_) => "function",
            Exp::Lambda(_) => "lambda",
        };
        Ok(Exp::Str(res.to_string()))
    }));

    Env { data, outer: None }
}

fn first(exps: &[Exp]) -> Result<Exp, Err> {
    match exps.get(0) {
        Some(exp) => Ok(exp.clone()),
        None => Err(Err::Reason("Expected first expression".to_string()))
    }
}

fn second(exps: &[Exp]) -> Result<Exp, Err> {
    match exps.get(1) {
        Some(exp) => Ok(exp.clone()),
        None => Err(Err::Reason("Expected second expression".to_string()))
    }
}

fn third(exps: &[Exp]) -> Result<Exp, Err> {
    match exps.get(2) {
        Some(exp) => Ok(exp.clone()),
        None => Err(Err::Reason("Expected third expression".to_string()))
    }
}

fn list_of_floats(args: &[Exp]) -> Result<Vec<f64>, Err> {
    args.iter().map(|x| float(x)).collect()
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
    first(args)
}

fn eval_atom_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    match eval(&first(args)?, env)? {
        Exp::List(_) => Ok(Exp::Bool(false)),
        _            => Ok(Exp::Bool(true)),
    }
}

fn eval_eq_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    match eval(&first(args)?, env)? {
        Exp::Str(a) => {
            match eval(&second(args)?, env)? {
                Exp::Str(b) => Ok(Exp::Bool(a == b)),
                _           => Ok(Exp::Bool(false)),
            }
        },
        Exp::Sym(a) => {
            match eval(&second(args)?, env)? {
                Exp::Sym(b) => Ok(Exp::Bool(a == b)),
                _           => Ok(Exp::Bool(false)),
            }
        },
        Exp::List(a) => {
            match eval(&second(args)?, env)? {
                Exp::List(b) => Ok(Exp::Bool(a.is_empty() && b.is_empty())),
                _            => Ok(Exp::Bool(false))
            }
        },
        _ => Ok(Exp::Bool(false))
    }
}

fn eval_car_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    match eval(&first(args)?, env)? {
        Exp::List(list) => {
            if list.is_empty() {
                return Err(Err::Reason("List cannot be empty".to_string())) // TODO: return nil?
            }
            Ok(list[0].clone())
        },
        _ => Err(Err::Reason("Expected list form".to_string())),
    }
}

fn eval_cdr_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    match eval(&first(args)?, env)? {
        Exp::List(list) => {
            if list.is_empty() {
                return Err(Err::Reason("List cannot be empty".to_string())) // TODO: return nil?
            }
            Ok(Exp::List(list[1..].to_vec()))
        },
        _ => Err(Err::Reason("Expected list form".to_string())),
    }
}

fn eval_cons_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    match eval(&second(args)?, env)? {
        Exp::List(mut list) => {
            list.insert(0, eval(&first(args)?, env)?);
            Ok(Exp::List(list.to_vec()))
        },
        _ => Err(Err::Reason("Expected list form".to_string())),
    }
}

fn eval_cond_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    if args.is_empty() {
        return Err(Err::Reason("Expected at least one form".to_string()))
    }
    for arg in args {
        match arg {
            Exp::List(list) => {
                if list.len() != 2 {
                    return Err(Err::Reason("Expected lists of predicate and expression".to_string()))
                }
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
    ensure_length!(args, 2);
    let id = match first(args)? {
        Exp::Sym(s) => Ok(s.clone()),
        _ => Err(Err::Reason("Expected first argument to be a symbol".to_string()))
    }?;
    let exp = eval(&second(args)?, env)?;
    env.data.insert(id.clone(), exp);
    Ok(Exp::Sym(id))
}

fn eval_lambda_args(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length!(args, 2);
    let params = first(args)?;
    let body = second(args)?;
    Ok(Exp::Lambda(Lambda {
        body: Rc::new(body.clone()),
        params: Rc::new(params.clone()),
    }))
}

fn eval_defun_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    ensure_length!(args, 3);
    let name = first(args)?;
    let params = second(args)?;
    let exp = third(args)?;
    let lambda_args = vec![Exp::Sym("lambda".to_string()), params, exp];
    let label_args = vec![name, Exp::List(lambda_args)];
    eval_label_args(&label_args, env)
}

fn eval_mapcar_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    ensure_length!(args, 2);
    match eval(&second(args)?, env) {
        Ok(Exp::List(list)) => {
            Ok(Exp::List(list.iter().map(|exp| {
                eval(&Exp::List(vec!(first(args)?, exp.clone())), env)
            }).collect::<Result<Vec<Exp>, Err>>()?))
        }
        _ => Err(Err::Reason("Expected second argument to be a list".to_string())),
    }
}

fn eval_load_args(args: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    let arg = first(args)?;
    let path = string(&arg)?;
    let mut code = fs::read_to_string(&path).or(Err(Err::Reason("Could not read file".to_string())))?;
    loop {
        let (rest, exp) = parse(&code)?;
        eval(&exp, env)?;
        if rest.is_empty() {
            break;
        }
        code = rest;
    }
    return Ok(Exp::Bool(true));
}

fn eval_built_in_form(exp: &Exp, args: &[Exp], env: &mut Env) -> Option<Result<Exp, Err>> {
    match exp {
        Exp::Sym(s) => {
            match s.as_ref() {
                // Seven Primitive Operators
                "quote"          => Some(eval_quote_args(args)),
                "atom" | "atom?" => Some(eval_atom_args(args, env)),
                "eq" | "eq?"     => Some(eval_eq_args(args, env)),
                "car" | "first"  => Some(eval_car_args(args, env)),
                "cdr" | "rest"   => Some(eval_cdr_args(args, env)),
                "cons"           => Some(eval_cons_args(args, env)),
                "cond"           => Some(eval_cond_args(args, env)),

                // Two Special Forms
                "label" | "def"  => Some(eval_label_args(args, env)),
                "lambda" | "fn"  => Some(eval_lambda_args(args)),

                "defun" | "defn" => Some(eval_defun_args(args, env)),
                "mapcar" | "map" => Some(eval_mapcar_args(args, env)),
                "load"           => Some(eval_load_args(args, env)),
                _                => None,
            }
        },
        _ => None,
    }
}

fn env_get(k: &str, env: &Env) -> Result<Exp, Err> {
    match env.data.get(k) {
        Some(exp) => Ok(exp.clone()),
        None => {
            match &env.outer {
                Some(outer_env) => env_get(k, outer_env),
                None => Err(Err::Reason(format!("Unexpected symbol '{}'", k))),
            }
        }
    }
}

fn list_of_symbols(form: Rc<Exp>) -> Result<Vec<String>, Err> {
    let list = match form.as_ref() {
        Exp::List(s) => Ok(s.clone()),
        _ => Err(Err::Reason("Expected args form to be a list".to_string()))
    }?;
    list.iter().map(|x| {
        match x {
            Exp::Sym(s) => Ok(s.clone()),
            _ => Err(Err::Reason("Expected symbols in the argument list".to_string()))
        }   
    }).collect()
}

fn env_for_lambda<'a>(params: Rc<Exp>, args: &[Exp], outer_env: &'a mut Env) -> Result<Env<'a>, Err> {
    let ks = list_of_symbols(params)?;
    if ks.len() != args.len() {
        let plural = if ks.len() == 1 { "" } else { "s" };
        return Err(Err::Reason(format!("Expected {} argument{}, got {}", ks.len(), plural, args.len())));
    }
    let vs = eval_forms(args, outer_env)?;
    let mut data: BTreeMap<String, Exp> = BTreeMap::new();
    for (k, v) in ks.iter().zip(vs.iter()) {
        data.insert(k.clone(), v.clone());
    }
    Ok(Env {
        data,
        outer: Some(outer_env),
    })
}

fn eval_forms(args: &[Exp], env: &mut Env) -> Result<Vec<Exp>, Err> {
    args.iter().map(|x| eval(x, env)).collect()
}

fn eval(exp: &Exp, env: &mut Env) -> Result<Exp, Err> {
    match exp {
        Exp::Sym(k) => env_get(k, env),
        Exp::Bool(_) => Ok(exp.clone()),
        Exp::Num(_) => Ok(exp.clone()),
        Exp::Str(_) => Ok(exp.clone()),
        Exp::List(list) => {
            let first_form = list.first().ok_or(Err::Reason("Expected a non-empty list".to_string()))?;
            let args = &list[1..];
            match eval_built_in_form(first_form, args, env) {
                Some(res) => res,
                None => {
                    let first_eval = eval(first_form, env)?;
                    match first_eval {
                        Exp::Func(func) => {
                            func(&eval_forms(args, env)?)
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


const COMPLETER_FORMS: [&str; 11] = [
    "quote", "atom?", "eq?", "first", "rest", "cons", "cond", "def", "fn",
    "defn", "print",
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
    match args.len() {
        1 => {
            repl(env)
        },
        2 => {
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
                eprintln!("File not found '{}'", pathname);
                usr::shell::ExitCode::CommandError
            }
        },
        _ => {
            usr::shell::ExitCode::CommandError
        },
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
}
