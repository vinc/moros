use super::FORMS;
use super::primitive;
use super::eval::BUILT_INS;
use super::eval::eval_args;
use super::{Err, Exp, Number};

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use core::borrow::Borrow;
use core::cell::RefCell;
use core::f64::consts::PI;

#[derive(Clone)]
pub struct Env {
    pub data: BTreeMap<String, Exp>,
    pub outer: Option<Rc<RefCell<Env>>>,
}

pub fn default_env() -> Rc<RefCell<Env>> {
    let mut data: BTreeMap<String, Exp> = BTreeMap::new();

    data.insert("pi".to_string(),                Exp::Num(Number::from(PI)));
    data.insert("=".to_string(),                 Exp::Primitive(primitive::lisp_eq));
    data.insert(">".to_string(),                 Exp::Primitive(primitive::lisp_gt));
    data.insert(">=".to_string(),                Exp::Primitive(primitive::lisp_gte));
    data.insert("<".to_string(),                 Exp::Primitive(primitive::lisp_lt));
    data.insert("<=".to_string(),                Exp::Primitive(primitive::lisp_lte));
    data.insert("*".to_string(),                 Exp::Primitive(primitive::lisp_mul));
    data.insert("+".to_string(),                 Exp::Primitive(primitive::lisp_add));
    data.insert("-".to_string(),                 Exp::Primitive(primitive::lisp_sub));
    data.insert("/".to_string(),                 Exp::Primitive(primitive::lisp_div));
    data.insert("%".to_string(),                 Exp::Primitive(primitive::lisp_mod));
    data.insert("^".to_string(),                 Exp::Primitive(primitive::lisp_exp));
    data.insert("<<".to_string(),                Exp::Primitive(primitive::lisp_shl));
    data.insert(">>".to_string(),                Exp::Primitive(primitive::lisp_shr));
    data.insert("cos".to_string(),               Exp::Primitive(primitive::lisp_cos));
    data.insert("acos".to_string(),              Exp::Primitive(primitive::lisp_acos));
    data.insert("asin".to_string(),              Exp::Primitive(primitive::lisp_asin));
    data.insert("atan".to_string(),              Exp::Primitive(primitive::lisp_atan));
    data.insert("sin".to_string(),               Exp::Primitive(primitive::lisp_sin));
    data.insert("tan".to_string(),               Exp::Primitive(primitive::lisp_tan));
    data.insert("trunc".to_string(),             Exp::Primitive(primitive::lisp_trunc));
    data.insert("system".to_string(),            Exp::Primitive(primitive::lisp_system));
    data.insert("read-file".to_string(),         Exp::Primitive(primitive::lisp_read_file));
    data.insert("read-file-bytes".to_string(),   Exp::Primitive(primitive::lisp_read_file_bytes));
    data.insert("write-file-bytes".to_string(),  Exp::Primitive(primitive::lisp_write_file_bytes));
    data.insert("append-file-bytes".to_string(), Exp::Primitive(primitive::lisp_append_file_bytes));
    data.insert("string".to_string(),            Exp::Primitive(primitive::lisp_string));
    data.insert("string->bytes".to_string(),     Exp::Primitive(primitive::lisp_string_bytes));
    data.insert("bytes->string".to_string(),     Exp::Primitive(primitive::lisp_bytes_string));
    data.insert("bytes->number".to_string(),     Exp::Primitive(primitive::lisp_bytes_number));
    data.insert("number->bytes".to_string(),     Exp::Primitive(primitive::lisp_number_bytes));
    data.insert("regex-find".to_string(),        Exp::Primitive(primitive::lisp_regex_find));
    data.insert("string->number".to_string(),    Exp::Primitive(primitive::lisp_string_number));
    data.insert("type".to_string(),              Exp::Primitive(primitive::lisp_type));
    data.insert("number-type".to_string(),       Exp::Primitive(primitive::lisp_number_type));
    data.insert("parse".to_string(),             Exp::Primitive(primitive::lisp_parse));
    data.insert("list".to_string(),              Exp::Primitive(primitive::lisp_list));
    data.insert("uniq".to_string(),              Exp::Primitive(primitive::lisp_uniq));
    data.insert("sort".to_string(),              Exp::Primitive(primitive::lisp_sort));
    data.insert("contains?".to_string(),         Exp::Primitive(primitive::lisp_contains));
    data.insert("slice".to_string(),             Exp::Primitive(primitive::lisp_slice));
    data.insert("chunks".to_string(),            Exp::Primitive(primitive::lisp_chunks));
    data.insert("split".to_string(),             Exp::Primitive(primitive::lisp_split));
    data.insert("trim".to_string(),              Exp::Primitive(primitive::lisp_trim));
    data.insert("length".to_string(),            Exp::Primitive(primitive::lisp_length));
    data.insert("append".to_string(),            Exp::Primitive(primitive::lisp_append));

    // Setup autocompletion
    *FORMS.lock() = data.keys().cloned().chain(BUILT_INS.map(String::from)).collect();

    Rc::new(RefCell::new(Env { data, outer: None }))
}

pub fn env_get(key: &str, env: &Rc<RefCell<Env>>) -> Result<Exp, Err> {
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

pub fn env_set(key: &str, val: Exp, env: &Rc<RefCell<Env>>) -> Result<Exp, Err> {
    let mut env = env.borrow_mut();
    match env.data.get(key) {
        Some(_) => {
            env.data.insert(key.to_string(), val.clone());
            Ok(val)
        }
        None => {
            match &env.outer {
                Some(outer_env) => env_set(key, val, outer_env.borrow()),
                None => Err(Err::Reason(format!("Unexpected symbol '{}'", key))),
            }
        }
    }
}

enum InnerEnv { Function, Macro }

fn inner_env(kind: InnerEnv, params: &Exp, args: &[Exp], outer: &mut Rc<RefCell<Env>>) -> Result<Rc<RefCell<Env>>, Err> {
    let mut args = match kind {
        InnerEnv::Function => eval_args(args, outer)?,
        InnerEnv::Macro => args.to_vec(),
    };
    let mut data: BTreeMap<String, Exp> = BTreeMap::new();
    match params {
        Exp::Sym(s) => {
            data.insert(s.clone(), Exp::List(args));
        }
        Exp::List(list) => {
            let mut list = list.to_vec();
            let n = list.len();
            let m = args.len();

            let mut is_variadic = false;
            if n > 0 {
                if let Exp::List(l) = &list[n - 1] {
                    if l.len() == 2 && l[0] == Exp::Sym("splice".to_string()) {
                        if let Exp::Sym(_) = &l[1] {
                            is_variadic = true;
                            list[n - 1] = l[1].clone();
                            if n <= m {
                                let rest = args.drain((n - 1)..).collect();
                                args.push(Exp::List(rest));
                            }
                        }
                    }
                }
            }
            let m = args.len();

            if n != m {
                let s = if n != 1 { "s" } else { "" };
                let a = if is_variadic { "at least " } else { "" };
                return Err(Err::Reason(format!("Expected {}{} argument{}, got {}", a, n, s, m)));
            }
            for (exp, arg) in list.iter().zip(args.iter()) {
                if let Exp::Sym(s) = exp {
                    data.insert(s.clone(), arg.clone());
                } else {
                    return Err(Err::Reason("Expected symbols in the argument list".to_string()));
                }
            }
        }
        _ => return Err(Err::Reason("Expected args form to be a list".to_string())),
    }
    Ok(Rc::new(RefCell::new(Env { data, outer: Some(Rc::new(RefCell::new(outer.borrow_mut().clone()))) })))
}

pub fn function_env(params: &Exp, args: &[Exp], outer: &mut Rc<RefCell<Env>>) -> Result<Rc<RefCell<Env>>, Err> {
    inner_env(InnerEnv::Function, params, args, outer)
}

pub fn macro_env(params: &Exp, args: &[Exp], outer: &mut Rc<RefCell<Env>>) -> Result<Rc<RefCell<Env>>, Err> {
    inner_env(InnerEnv::Macro, params, args, outer)
}
