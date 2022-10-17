use super::{Err, Exp, Number};
use super::FORMS;
use super::eval::BUILT_INS;
use super::parse::parse;
use super::eval::eval_args;
use super::{float, number, string};
use super::list_of_numbers;
use super::list_of_symbols;
use super::list_of_bytes;

use crate::usr::shell;
use crate::api::fs;
use crate::api::regex::Regex;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use core::borrow::Borrow;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::f64::consts::PI;
use core::convert::TryInto;

#[derive(Clone)]
pub struct Env {
    pub data: BTreeMap<String, Exp>,
    pub outer: Option<Rc<RefCell<Env>>>,
}

#[macro_export]
macro_rules! ensure_length_eq {
    ($list:expr, $count:expr) => {
        if $list.len() != $count {
            let plural = if $count != 1 { "s" } else { "" };
            return Err(Err::Reason(format!("Expected {} expression{}", $count, plural)))
        }
    };
}

#[macro_export]
macro_rules! ensure_length_gt {
    ($list:expr, $count:expr) => {
        if $list.len() <= $count {
            let plural = if $count != 1 { "s" } else { "" };
            return Err(Err::Reason(format!("Expected more than {} expression{}", $count, plural)))
        }
    };
}

pub fn default_env() -> Rc<RefCell<Env>> {
    let mut data: BTreeMap<String, Exp> = BTreeMap::new();
    data.insert("pi".to_string(), Exp::Num(Number::from(PI)));
    data.insert("=".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        Ok(Exp::Bool(list_of_numbers(args)?.windows(2).all(|nums| nums[0] == nums[1])))
    }));
    data.insert(">".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        Ok(Exp::Bool(list_of_numbers(args)?.windows(2).all(|nums| nums[0] > nums[1])))
    }));
    data.insert(">=".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        Ok(Exp::Bool(list_of_numbers(args)?.windows(2).all(|nums| nums[0] >= nums[1])))
    }));
    data.insert("<".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        Ok(Exp::Bool(list_of_numbers(args)?.windows(2).all(|nums| nums[0] < nums[1])))
    }));
    data.insert("<=".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        Ok(Exp::Bool(list_of_numbers(args)?.windows(2).all(|nums| nums[0] <= nums[1])))
    }));
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
        let mut args = list_of_numbers(args)?;
        if args.len() == 1 {
            args.insert(0, Number::Int(1));
        }
        for arg in &args[1..] {
            if arg.is_zero() {
                return Err(Err::Reason("Division by zero".to_string()));
            }
        }
        let car = args[0].clone();
        let res = args[1..].iter().fold(car, |acc, a| acc / a.clone());
        Ok(Exp::Num(res))
    }));
    data.insert("%".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_gt!(args, 0);
        let args = list_of_numbers(args)?;
        for arg in &args[1..] {
            if arg.is_zero() {
                return Err(Err::Reason("Division by zero".to_string()));
            }
        }
        let car = args[0].clone();
        let res = args[1..].iter().fold(car, |acc, a| acc % a.clone());
        Ok(Exp::Num(res))
    }));
    data.insert("^".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_gt!(args, 0);
        let args = list_of_numbers(args)?;
        let car = args[0].clone();
        let res = args[1..].iter().fold(car, |acc, a| acc.pow(a));
        Ok(Exp::Num(res))
    }));
    data.insert("<<".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 2);
        let args = list_of_numbers(args)?;
        let res = args[0].clone() << args[1].clone();
        Ok(Exp::Num(res))
    }));
    data.insert(">>".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 2);
        let args = list_of_numbers(args)?;
        let res = args[0].clone() >> args[1].clone();
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
        match shell::exec(&cmd) {
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
        let len = number(&args[1])?;
        let mut buf = vec![0; len.try_into()?];
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
        ensure_length_eq!(args, 2);
        match (&args[0], &args[1]) { // TODO: default type to "int" and make it optional
            (Exp::List(list), Exp::Str(kind)) => {
                let bytes = list_of_bytes(list)?;
                ensure_length_eq!(bytes, 8);
                match kind.as_str() { // TODO: bigint
                    "int" => Ok(Exp::Num(Number::Int(i64::from_be_bytes(bytes[0..8].try_into().unwrap())))),
                    "float" => Ok(Exp::Num(Number::Float(f64::from_be_bytes(bytes[0..8].try_into().unwrap())))),
                    _ => Err(Err::Reason("Invalid number type".to_string())),
                }
            }
            _ => Err(Err::Reason("Expected args to be the number type and a list of bytes".to_string()))
        }

    }));
    data.insert("number->bytes".to_string(), Exp::Primitive(|args: &[Exp]| -> Result<Exp, Err> {
        ensure_length_eq!(args, 1);
        let n = number(&args[0])?;
        Ok(Exp::List(n.to_be_bytes().iter().map(|b| Exp::Num(Number::from(*b))).collect()))
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

pub fn env_set(key: &str, val: Exp, env: &Rc<RefCell<Env>>) -> Result<(), Err> {
    let mut env = env.borrow_mut();
    match env.data.get(key) {
        Some(_) => {
            env.data.insert(key.to_string(), val);
            Ok(())
        }
        None => {
            match &env.outer {
                Some(outer_env) => env_set(key, val, outer_env.borrow()),
                None => Err(Err::Reason(format!("Unexpected symbol '{}'", key))),
            }
        }
    }
}

pub fn lambda_env(params: Rc<Exp>, args: &[Exp], outer: &mut Rc<RefCell<Env>>) -> Result<Rc<RefCell<Env>>, Err> {
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
