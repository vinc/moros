use super::{Err, Exp, Env, Function, parse_eval};
use super::env::{env_keys, env_get, env_set, function_env};
use super::expand::expand;
use crate::could_not;
use super::string;

use crate::{ensure_length_eq, ensure_length_gt, expected};
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

fn eval_equal_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let a = eval(&args[0], env)?;
    let b = eval(&args[1], env)?;
    Ok(Exp::Bool(a == b))
}

fn eval_head_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::List(list) => {
            ensure_length_gt!(list, 0);
            Ok(list[0].clone())
        },
        _ => expected!("first argument to be a list"),
    }
}

fn eval_tail_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::List(list) => {
            ensure_length_gt!(list, 0);
            Ok(Exp::List(list[1..].to_vec()))
        },
        _ => expected!("first argument to be a list"),
    }
}

fn eval_cons_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match eval(&args[1], env)? {
        Exp::List(mut list) => {
            list.insert(0, eval(&args[0], env)?);
            Ok(Exp::List(list))
        },
        _ => expected!("first argument to be a list"),
    }
}

pub fn eval_variable_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match &args[0] {
        Exp::Sym(name) => {
            let exp = eval(&args[1], env)?;
            env.borrow_mut().data.insert(name.clone(), exp);
            Ok(Exp::Sym(name.clone()))
        }
        _ => expected!("first argument to be a symbol")
    }
}

fn eval_set_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match &args[0] {
        Exp::Sym(name) => {
            let exp = eval(&args[1], env)?;
            Ok(env_set(name, exp, env)?)
        }
        _ => expected!("first argument to be a symbol")
    }
}

fn eval_env_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 0);
    let keys = env_keys(env)?.iter().map(|k| Exp::Sym(k.clone())).collect();
    Ok(Exp::List(keys))
}

fn eval_while_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_gt!(args, 1);
    let cond = &args[0];
    let mut res = Exp::List(vec![]);
    while eval(cond, env)?.is_truthy() {
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
        _ => return expected!("last argument to be a list"),
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
    let path = string(&eval(&args[0], env)?)?;
    let mut input = fs::read_to_string(&path).or(could_not!("find file '{}'", path))?;
    loop {
        let (rest, _) = parse_eval(&input, env)?;
        if rest.is_empty() {
            break;
        }
        input = rest;
    }
    Ok(Exp::Bool(true))
}

fn eval_doc_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::Primitive(_) => Ok(Exp::Str("".to_string())),
        Exp::Function(f) => Ok(Exp::Str(f.doc.unwrap_or("".to_string()))),
        Exp::Macro(m) => Ok(Exp::Str(m.doc.unwrap_or("".to_string()))),
        _ => expected!("function or macro"),
    }
}

pub fn eval_args(args: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Vec<Exp>, Err> {
    args.iter().map(|x| eval(x, env)).collect()
}

pub const BUILT_INS: [&str; 26] = [
    "quote", "quasiquote", "unquote", "unquote-splicing",
    "atom?", "equal?", "head", "tail", "cons",
    "if", "cond", "while",
    "variable", "function", "macro",
    "define-function", "define",
    "define-macro",
    "set",
    "apply", "eval", "expand",
    "do",
    "load",
    "doc",
    "env"
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
                    Exp::Sym(s) if s == "quote"    => return eval_quote_args(args),
                    Exp::Sym(s) if s == "atom?"    => return eval_atom_args(args, env),
                    Exp::Sym(s) if s == "equal?"   => return eval_equal_args(args, env),
                    Exp::Sym(s) if s == "head"     => return eval_head_args(args, env),
                    Exp::Sym(s) if s == "tail"     => return eval_tail_args(args, env),
                    Exp::Sym(s) if s == "cons"     => return eval_cons_args(args, env),
                    Exp::Sym(s) if s == "set"      => return eval_set_args(args, env),
                    Exp::Sym(s) if s == "while"    => return eval_while_args(args, env),
                    Exp::Sym(s) if s == "apply"    => return eval_apply_args(args, env),
                    Exp::Sym(s) if s == "eval"     => return eval_eval_args(args, env),
                    Exp::Sym(s) if s == "do"       => return eval_do_args(args, env),
                    Exp::Sym(s) if s == "load"     => return eval_load_args(args, env),
                    Exp::Sym(s) if s == "doc"      => return eval_doc_args(args, env),
                    Exp::Sym(s) if s == "variable" => return eval_variable_args(args, env),
                    Exp::Sym(s) if s == "env"      => return eval_env_args(args, env),
                    Exp::Sym(s) if s == "expand"   => {
                        ensure_length_eq!(args, 1);
                        return expand(&args[0], env);
                    }
                    Exp::Sym(s) if s == "if" => {
                        ensure_length_gt!(args, 1);
                        if eval(&args[0], env)?.is_truthy() { // consequent
                            exp_tmp = args[1].clone();
                        } else if args.len() > 2 { // alternate
                            exp_tmp = args[2].clone();
                        } else { // '()
                            exp_tmp = Exp::List(vec![Exp::Sym("quote".to_string()), Exp::List(vec![])]);
                        }
                        exp = &exp_tmp;
                    }
                    Exp::Sym(s) if s == "function" || s == "macro" => {
                        let (params, body, doc) = match args.len() {
                            2 => (args[0].clone(), args[1].clone(), None),
                            3 => (args[0].clone(), args[2].clone(), Some(string(&args[1])?)),
                            _ => return expected!("3 or 4 arguments"),
                        };
                        let f = Box::new(Function { params, body, doc });
                        let exp = if s == "function" { Exp::Function(f) } else { Exp::Macro(f) };
                        return Ok(exp);
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
                            _ => return expected!("first argument to be a function"),
                        }
                    }
                }
            },
            _ => return Err(Err::Reason("Unexpected argument".to_string())),
        }
    }
}
