use super::{Err, Exp, Env, Lambda};
use super::env::{env_get, env_set, lambda_env};
use super::parse::parse;
use super::string;

use crate::{ensure_length_eq, ensure_length_gt};
use crate::api::fs;

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
                match eval(&list[0], env)? {
                    Exp::Bool(b) if b => return eval(&list[1], env),
                    _ => continue,
                }
            },
            _ => return Err(Err::Reason("Expected lists of predicate and expression".to_string())),
        }
    }
    Ok(Exp::List(vec![]))
}

fn eval_if_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_gt!(args, 1);
    if eval(&args[0], env)? == Exp::Bool(true) {
        eval(&args[1], env)
    } else if args.len() > 2 {
        eval(&args[2], env)
    } else {
        Ok(Exp::List(vec![]))
    }
}

pub fn eval_label_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
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

pub const BUILT_INS: [&str; 24] = [
    "quote", "atom", "eq", "car", "cdr", "cons", "cond", "label", "lambda", "define", "def",
    "function", "fun", "fn", "if", "while", "defun", "defn", "apply", "eval", "progn", "begin", "do",
    "load"
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

                "if"                         => Some(eval_if_args(args, env)),
                "set"                        => Some(eval_set_args(args, env)),
                "while"                      => Some(eval_while_args(args, env)),
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

pub fn eval_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Vec<Exp>, Err> {
    args.iter().map(|x| eval(x, env)).collect()
}

pub fn eval(exp: &Exp, env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
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
