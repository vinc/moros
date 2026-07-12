use super::env::{env_get, env_keys, env_set, bind};
use super::expand::expand;
use super::string;
use super::{exec, Env, Err, Exp, Function};

use crate::{ensure_length_eq, ensure_length_gt, expected};

use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp::Ordering::Equal;

fn eval_quote_args(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(args[0].clone())
}

fn eval_atom_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::List(_) => Ok(Exp::Bool(false)),
        _ => Ok(Exp::Bool(true)),
    }
}

fn eval_equal_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_gt!(args, 1);

    let exps: Vec<Exp> = args.iter().map(|arg|
        eval(&arg, env)
    ).collect::<Result<_,_>>()?;

    Ok(Exp::Bool(exps.windows(2).all(|pair|
        match (&pair[0], &pair[1]) {
            (Exp::Num(a), Exp::Num(b)) => a.partial_cmp(b) == Some(Equal),
            (a, b) => a == b,
        }
    )))
}

fn eval_cons_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let exp = eval(&args[0], env)?;
    match eval(&args[1], env)? {
        Exp::List(mut list) => {
            list.insert(0, exp);
            Ok(Exp::List(list))
        }
        _ => expected!("first argument to be a list"),
    }
}

fn eval_is_variable_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match &args[0] {
        Exp::Sym(name) => {
            Ok(Exp::Bool(env_get(name, env).is_ok()))
        }
        _ => expected!("first argument to be a symbol"),
    }
}

pub fn eval_variable_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match &args[0] {
        Exp::Sym(name) => {
            let exp = eval(&args[1], env)?;
            env.borrow_mut().data.insert(name.clone(), exp);
            Ok(Exp::Sym(name.clone()))
        }
        _ => expected!("first argument to be a symbol"),
    }
}

fn eval_mutate_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match &args[0] {
        Exp::Sym(name) => {
            let exp = eval(&args[1], env)?;
            Ok(env_set(name, exp, env)?)
        }
        _ => expected!("first argument to be a symbol"),
    }
}

fn eval_env_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_eq!(args, 0);
    let keys = env_keys(env)?.iter().map(|k| Exp::Sym(k.clone())).collect();
    Ok(Exp::List(keys))
}

fn eval_apply_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_gt!(args, 1);
    let i = args.len() - 1;
    let last = args[i].clone();
    let mut args = eval_args(&args[0..i], env)?;
    match eval(&last, env)? {
        Exp::List(rest) => args.extend(rest),
        _ => return expected!("last argument to be a list"),
    }
    apply(&args[0], &args[1..], env)
}

fn eval_while_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
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

fn eval_fold_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_eq!(args, 3);
    let fun = eval(&args[0], env)?;
    let mut acc = eval(&args[1], env)?;
    match eval(&args[2], env)? {
        Exp::List(list) => {
            for arg in list {
                acc = apply(&fun, &[acc, arg], env)?;
            }
        }
        _ => return expected!("last argument to be a list"),
    }
    Ok(acc)
}

fn eval_eval_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let exp = eval(&args[0], env)?;
    eval(&exp, env)
}

fn eval_do_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    let mut res = Ok(Exp::List(vec![]));
    for arg in args {
        res = Ok(eval(arg, env)?);
    }
    res
}

fn eval_load_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let path = string(&eval(&args[0], env)?)?;
    exec(&path, env)?;
    Ok(Exp::Bool(true))
}

fn eval_doc_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match eval(&args[0], env)? {
        Exp::Primitive(_) => Ok(Exp::Str("".to_string())),
        Exp::Function(f) => Ok(Exp::Str(f.doc.unwrap_or("".to_string()))),
        Exp::Macro(m) => Ok(Exp::Str(m.doc.unwrap_or("".to_string()))),
        _ => expected!("function or macro"),
    }
}

pub fn eval_args(
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Vec<Exp>, Err> {
    args.iter().map(|x| eval(x, env)).collect()
}

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
            Exp::Dict(_) => return Ok(exp.clone()),
            Exp::List(list) => {
                ensure_length_gt!(list, 0);
                let args = &list[1..];
                match &list[0] {
                    Exp::Sym(s) if s == "quote" => {
                        return eval_quote_args(args);
                    }
                    Exp::Sym(s) if s == "atom?" => {
                        return eval_atom_args(args, env);
                    }
                    Exp::Sym(s) if s == "eq?" => {
                        return eval_equal_args(args, env);
                    }
                    Exp::Sym(s) if s == "cons" => {
                        return eval_cons_args(args, env);
                    }
                    Exp::Sym(s) if s == "while" => {
                        return eval_while_args(args, env);
                    }
                    Exp::Sym(s) if s == "eval" => {
                        return eval_eval_args(args, env);
                    }
                    Exp::Sym(s) if s == "do" => {
                        return eval_do_args(args, env);
                    }
                    Exp::Sym(s) if s == "load" => {
                        return eval_load_args(args, env);
                    }
                    Exp::Sym(s) if s == "doc" => {
                        return eval_doc_args(args, env);
                    }
                    Exp::Sym(s) if s == "var?" => {
                        return eval_is_variable_args(args, env);
                    }
                    Exp::Sym(s) if s == "var" => {
                        return eval_variable_args(args, env);
                    }
                    Exp::Sym(s) if s == "mut" => {
                        return eval_mutate_args(args, env);
                    }
                    Exp::Sym(s) if s == "env" => {
                        return eval_env_args(args, env);
                    }
                    Exp::Sym(s) if s == "fold" => {
                        return eval_fold_args(args, env);
                    }
                    Exp::Sym(s) if s == "apply" => {
                        return eval_apply_args(args, env);
                    }
                    Exp::Sym(s) if s == "expand" => {
                        ensure_length_eq!(args, 1);
                        return expand(&args[0], env);
                    }
                    Exp::Sym(s) if s == "if" => {
                        ensure_length_gt!(args, 1);
                        if eval(&args[0], env)?.is_truthy() { // Consequent
                            exp_tmp = args[1].clone();
                        } else if args.len() > 2 { // Alternate
                            exp_tmp = args[2].clone();
                        } else { // '()
                            exp_tmp = Exp::List(vec![
                                Exp::Sym("quote".to_string()),
                                Exp::List(vec![]),
                            ]);
                        }
                        exp = &exp_tmp;
                    }
                    Exp::Sym(s) if s == "fun" || s == "mac" => {
                        let (params, body, doc) = match args.len() {
                            2 => {
                                (args[0].clone(), args[1].clone(), None)
                            }
                            3 => {
                                let doc = Some(string(&args[1])?);
                                (args[0].clone(), args[2].clone(), doc)
                            }
                            _ => return expected!("3 or 4 arguments"),
                        };
                        let f = Box::new(Function { params, body, doc });
                        let exp = if s == "fun" {
                            Exp::Function(f)
                        } else {
                            Exp::Macro(f)
                        };
                        return Ok(exp);
                    }
                    _ => {
                        let f = eval(&list[0], env)?;
                        let args = eval_args(args, env)?;
                        match f {
                            Exp::Function(f) => {
                                env_tmp = bind(&f.params, &args, env)?;
                                exp_tmp = f.body;
                                env = &mut env_tmp;
                                exp = &exp_tmp;
                            }
                            Exp::Primitive(f) => {
                                return f(&args);
                            }
                            _ => {
                                return expected!(
                                    "first argument to be a function"
                                );
                            }
                        }
                    },
                }
            }
            _ => return Err(Err::Reason("Unexpected argument".to_string())),
        }
    }
}

fn apply(
    f: &Exp,
    args: &[Exp],
    env: &mut Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    match f {
        Exp::Function(f) => {
            let mut inner_env = bind(&f.params, &args, env)?;
            eval(&f.body, &mut inner_env)
        }
        Exp::Primitive(f) => {
            f(&args)
        }
        _ => {
            expected!("first argument to be a function")
        }
    }
}
