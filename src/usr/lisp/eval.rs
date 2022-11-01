use super::{Err, Exp, Env, Function};
use super::env::{env_get, env_set, function_env};
use super::parse::parse;
use super::expand::expand;
use super::string;

use crate::{ensure_length_eq, ensure_length_gt};
use crate::api::fs;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;
use core::cell::RefCell;

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
            Ok(Exp::List(list))
        },
        _ => Err(Err::Reason("Expected list form".to_string())),
    }
}

pub fn eval_define_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match &args[0] {
        Exp::Sym(name) => {
            let exp = eval(&args[1], env)?;
            env.borrow_mut().data.insert(name.clone(), exp);
            Ok(Exp::Sym(name.clone()))
        }
        _ => Err(Err::Reason("Expected first argument to be a symbol".to_string()))
    }
}

fn eval_set_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match &args[0] {
        Exp::Sym(name) => {
            let exp = eval(&args[1], env)?;
            env_set(name, exp, env)?;
            Ok(Exp::Sym(name.clone()))
        }
        _ => Err(Err::Reason("Expected first argument to be a symbol".to_string()))
    }
}

fn eval_while_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_gt!(args, 1);
    let cond = &args[0];
    let mut res = Exp::List(vec![]);
    while eval(cond, env)? == Exp::Bool(true) {
        for arg in &args[1..] {
            res = eval(arg, env)?;
        }
    }
    Ok(res)
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

fn eval_do_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
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
        let exp = expand(&exp, env)?;
        eval(&exp, env)?;
        if rest.is_empty() {
            break;
        }
        code = rest;
    }
    Ok(Exp::Bool(true))
}

pub fn eval_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Vec<Exp>, Err> {
    args.iter().map(|x| eval(x, env)).collect()
}

pub const BUILT_INS: [&str; 32] = [
    "quote", "quasiquote", "unquote", "unquote-splicing",
    "atom", "eq", "car", "cdr", "cons",
    "if", "cond", "while",
    "set",
    "define", "def", "label",
    "function", "fun", "lambda",
    "macro", "mac",
    "define-function", "def-fun",
    "define-macro", "def-mac",
    "apply", "eval", "expand",
    "do", "begin", "progn",
    "load"
];

pub fn eval(exp: &Exp, env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    let mut exp = exp;
    let mut env = env;
    let mut env_tmp;
    let mut exp_tmp;
    loop {
        match exp {
            Exp::Sym(key) => return env_get(key, env),
            Exp::Bool(_) => return Ok(exp.clone()),
            Exp::Num(_) => return Ok(exp.clone()),
            Exp::Str(_) => return Ok(exp.clone()),
            Exp::List(list) => {
                ensure_length_gt!(list, 0);
                let args = &list[1..];
                match &list[0] {
                    Exp::Sym(s) if s == "quote"  => return eval_quote_args(args),
                    Exp::Sym(s) if s == "atom"   => return eval_atom_args(args, env),
                    Exp::Sym(s) if s == "eq"     => return eval_eq_args(args, env),
                    Exp::Sym(s) if s == "car"    => return eval_car_args(args, env),
                    Exp::Sym(s) if s == "cdr"    => return eval_cdr_args(args, env),
                    Exp::Sym(s) if s == "cons"   => return eval_cons_args(args, env),
                    Exp::Sym(s) if s == "set"    => return eval_set_args(args, env),
                    Exp::Sym(s) if s == "while"  => return eval_while_args(args, env),
                    Exp::Sym(s) if s == "apply"  => return eval_apply_args(args, env),
                    Exp::Sym(s) if s == "eval"   => return eval_eval_args(args, env),
                    Exp::Sym(s) if s == "do"     => return eval_do_args(args, env),
                    Exp::Sym(s) if s == "load"   => return eval_load_args(args, env),
                    Exp::Sym(s) if s == "define" => return eval_define_args(args, env),
                    Exp::Sym(s) if s == "expand" => {
                        ensure_length_eq!(args, 1);
                        return expand(&args[0], env);
                    }
                    Exp::Sym(s) if s == "if" => {
                        ensure_length_gt!(args, 1);
                        if eval(&args[0], env)? == Exp::Bool(true) { // consequent
                            exp_tmp = args[1].clone();
                        } else if args.len() > 2 { // alternate
                            exp_tmp = args[2].clone();
                        } else { // '()
                            exp_tmp = Exp::List(vec![Exp::Sym("quote".to_string()), Exp::List(vec![])]);
                        }
                        exp = &exp_tmp;
                    }
                    Exp::Sym(s) if s == "function" => {
                        ensure_length_eq!(args, 2);
                        return Ok(Exp::Function(Box::new(Function {
                            params: args[0].clone(),
                            body: args[1].clone(),
                        })))
                    }
                    Exp::Sym(s) if s == "macro" => {
                        ensure_length_eq!(args, 2);
                        return Ok(Exp::Macro(Box::new(Function {
                            params: args[0].clone(),
                            body: args[1].clone(),
                        })))
                    }
                    _ => {
                        match eval(&list[0], env)? {
                            Exp::Function(f) => {
                                env_tmp = function_env(&f.params, args, env)?;
                                exp_tmp = f.body;
                                env = &mut env_tmp;
                                exp = &exp_tmp;
                            },
                            Exp::Primitive(f) => {
                                return f(&eval_args(args, env)?)
                            },
                            _ => return Err(Err::Reason("First form must be a function".to_string())),
                        }
                    }
                }
            },
            _ => return Err(Err::Reason("Unexpected form".to_string())),
        }
    }
}
