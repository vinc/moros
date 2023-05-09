use super::{Err, Exp, Env};
use super::env::{env_get, macro_env};
use super::eval::eval;

use crate::{ensure_length_eq, ensure_length_gt};

use alloc::format;
use alloc::rc::Rc;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;
use core::cell::RefCell;

pub fn expand_quasiquote(exp: &Exp) -> Result<Exp, Err> {
    match exp {
        Exp::List(list) if list.len() > 0 => {
            match &list[0] {
                Exp::Sym(s) if s == "unquote" => {
                    Ok(list[1].clone())
                }
                Exp::List(l) if l.len() == 2 && l[0] == Exp::Sym("unquote-splice".to_string()) => {
                    Ok(Exp::List(vec![
                        Exp::Sym("append".to_string()),
                        l[1].clone(),
                        expand_quasiquote(&Exp::List(list[1..].to_vec()))?
                    ]))
                }
                _ => {
                    Ok(Exp::List(vec![
                        Exp::Sym("cons".to_string()),
                        expand_quasiquote(&list[0])?,
                        expand_quasiquote(&Exp::List(list[1..].to_vec()))?,
                    ]))
                }
            }
        }
        _ => Ok(Exp::List(vec![Exp::Sym("quote".to_string()), exp.clone()])),
    }
}

pub fn expand_list(list: &[Exp], env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    let expanded: Result<Vec<Exp>, Err> = list.iter().map(|item| expand(item, env)).collect();
    Ok(Exp::List(expanded?))
}

pub fn expand(exp: &Exp, env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    if let Exp::List(list) = exp {
        ensure_length_gt!(list, 0);
        match &list[0] {
            Exp::Sym(s) if s == "quote" => {
                ensure_length_eq!(list, 2);
                Ok(exp.clone())
            }
            Exp::Sym(s) if s == "quasiquote" => {
                ensure_length_eq!(list, 2);
                expand_quasiquote(&list[1])
            }
            Exp::Sym(s) if s == "define-function" || s == "define" => {
                ensure_length_eq!(list, 3);
                match (&list[1], &list[2]) {
                    (Exp::List(args), Exp::List(_)) => {
                        ensure_length_gt!(args, 0);
                        let name = args[0].clone();
                        let args = Exp::List(args[1..].to_vec());
                        let body = expand(&list[2], env)?;
                        Ok(Exp::List(vec![
                            Exp::Sym("variable".to_string()), name, Exp::List(vec![
                                Exp::Sym("function".to_string()), args, body
                            ])
                        ]))
                    }
                    (Exp::Sym(_), _) => expand_list(list, env),
                    _ => Err(Err::Reason("Expected first argument to be a symbol or a list".to_string()))
                }
            }
            Exp::Sym(s) if s == "define-macro" => {
                ensure_length_eq!(list, 3);
                match (&list[1], &list[2]) {
                    (Exp::List(args), Exp::List(_)) => {
                        ensure_length_gt!(args, 0);
                        let name = args[0].clone();
                        let args = Exp::List(args[1..].to_vec());
                        let body = expand(&list[2], env)?;
                        Ok(Exp::List(vec![
                            Exp::Sym("variable".to_string()), name, Exp::List(vec![
                                Exp::Sym("macro".to_string()), args, body
                            ])
                        ]))
                    }
                    (Exp::Sym(_), _) => expand_list(list, env),
                    _ => Err(Err::Reason("Expected first argument to be a symbol or a list".to_string()))
                }
            }
            Exp::Sym(s) if s == "cond" => {
                ensure_length_gt!(list, 1);
                if let Exp::List(args) = &list[1] {
                    ensure_length_eq!(args, 2);
                    let mut res = vec![Exp::Sym("if".to_string()), args[0].clone(), args[1].clone()];
                    if list.len() > 2 {
                        let mut acc = vec![Exp::Sym("cond".to_string())];
                        acc.extend_from_slice(&list[2..]);
                        res.push(expand(&Exp::List(acc), env)?);
                    }
                    Ok(Exp::List(res))
                } else {
                    Err(Err::Reason("Expected lists of predicate and expression".to_string()))
                }
            }
            Exp::Sym(s) => {
                if let Ok(Exp::Macro(m)) = env_get(s, env) {
                    let mut m_env = macro_env(&m.params, &list[1..], env)?;
                    let m_exp = m.body;
                    expand(&eval(&m_exp, &mut m_env)?, env)
                } else {
                    expand_list(list, env)
                }
            }
            _ => expand_list(list, env),
        }
    } else {
        Ok(exp.clone())
    }
}
