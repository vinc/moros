use super::parse::parse;
use super::{Err, Exp, Number};
use super::{float, number, string};
use super::{bytes, numbers, strings};

use crate::{ensure_length_eq, ensure_length_gt};
use crate::api::fs;
use crate::api::regex::Regex;
use crate::usr::shell;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;
use core::cmp::Ordering::Equal;
use core::convert::TryFrom;
use core::convert::TryInto;

pub fn lisp_eq(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::Bool(numbers(args)?.windows(2).all(|nums| nums[0] == nums[1])))
}

pub fn lisp_gt(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::Bool(numbers(args)?.windows(2).all(|nums| nums[0] > nums[1])))
}

pub fn lisp_gte(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::Bool(numbers(args)?.windows(2).all(|nums| nums[0] >= nums[1])))
}

pub fn lisp_lt(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::Bool(numbers(args)?.windows(2).all(|nums| nums[0] < nums[1])))
}

pub fn lisp_lte(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::Bool(numbers(args)?.windows(2).all(|nums| nums[0] <= nums[1])))
}

pub fn lisp_mul(args: &[Exp]) -> Result<Exp, Err> {
    let res = numbers(args)?.iter().fold(Number::Int(1), |acc, a| acc * a.clone());
    Ok(Exp::Num(res))
}

pub fn lisp_add(args: &[Exp]) -> Result<Exp, Err> {
    let res = numbers(args)?.iter().fold(Number::Int(0), |acc, a| acc + a.clone());
    Ok(Exp::Num(res))
}

pub fn lisp_sub(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    let args = numbers(args)?;
    let head = args[0].clone();
    if args.len() == 1 {
        Ok(Exp::Num(-head))
    } else {
        let res = args[1..].iter().fold(Number::Int(0), |acc, a| acc + a.clone());
        Ok(Exp::Num(head - res))
    }
}

pub fn lisp_div(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    let mut args = numbers(args)?;
    if args.len() == 1 {
        args.insert(0, Number::Int(1));
    }
    for arg in &args[1..] {
        if arg.is_zero() {
            return Err(Err::Reason("Division by zero".to_string()));
        }
    }
    let head = args[0].clone();
    let res = args[1..].iter().fold(head, |acc, a| acc / a.clone());
    Ok(Exp::Num(res))
}

pub fn lisp_mod(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    let args = numbers(args)?;
    for arg in &args[1..] {
        if arg.is_zero() {
            return Err(Err::Reason("Division by zero".to_string()));
        }
    }
    let head = args[0].clone();
    let res = args[1..].iter().fold(head, |acc, a| acc % a.clone());
    Ok(Exp::Num(res))
}

pub fn lisp_exp(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    let args = numbers(args)?;
    let head = args[0].clone();
    let res = args[1..].iter().fold(head, |acc, a| acc.pow(a));
    Ok(Exp::Num(res))
}

pub fn lisp_shl(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let args = numbers(args)?;
    let res = args[0].clone() << args[1].clone();
    Ok(Exp::Num(res))
}

pub fn lisp_shr(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let args = numbers(args)?;
    let res = args[0].clone() >> args[1].clone();
    Ok(Exp::Num(res))
}

pub fn lisp_cos(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(Exp::Num(number(&args[0])?.cos()))
}

pub fn lisp_acos(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    if -1.0 <= float(&args[0])? && float(&args[0])? <= 1.0 {
        Ok(Exp::Num(number(&args[0])?.acos()))
    } else {
        Err(Err::Reason("Expected argument to be between -1.0 and 1.0".to_string()))
    }
}

pub fn lisp_asin(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    if -1.0 <= float(&args[0])? && float(&args[0])? <= 1.0 {
        Ok(Exp::Num(number(&args[0])?.asin()))
    } else {
        Err(Err::Reason("Expected argument to be between -1.0 and 1.0".to_string()))
    }
}

pub fn lisp_atan(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(Exp::Num(number(&args[0])?.atan()))
}

pub fn lisp_sin(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(Exp::Num(number(&args[0])?.sin()))
}

pub fn lisp_tan(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(Exp::Num(number(&args[0])?.tan()))
}

pub fn lisp_trunc(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    Ok(Exp::Num(number(&args[0])?.trunc()))
}

pub fn lisp_system(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_gt!(args, 0);
    let cmd = strings(&args)?.join(" ");
    match shell::exec(&cmd) {
        Ok(()) => Ok(Exp::Num(Number::from(0 as u8))),
        Err(code) => Ok(Exp::Num(Number::from(code as u8))),
    }
}

pub fn lisp_read_file(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let path = string(&args[0])?;
    let contents = fs::read_to_string(&path).or(Err(Err::Reason("Could not read file".to_string())))?;
    Ok(Exp::Str(contents))
}

pub fn lisp_read_file_bytes(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let path = string(&args[0])?;
    let len = number(&args[1])?;
    let mut buf = vec![0; len.try_into()?];
    let n = fs::read(&path, &mut buf).or(Err(Err::Reason("Could not read file".to_string())))?;
    buf.resize(n, 0);
    Ok(Exp::List(buf.iter().map(|b| Exp::Num(Number::from(*b))).collect()))
}

pub fn lisp_write_file_bytes(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let path = string(&args[0])?;
    match &args[1] {
        Exp::List(list) => {
            let buf = bytes(list)?;
            let n = fs::write(&path, &buf).or(Err(Err::Reason("Could not write file".to_string())))?;
            Ok(Exp::Num(Number::from(n)))
        }
        _ => Err(Err::Reason("Expected second argument to be a list".to_string()))
    }
}

pub fn lisp_append_file_bytes(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let path = string(&args[0])?;
    match &args[1] {
        Exp::List(list) => {
            let buf = bytes(list)?;
            let n = fs::append(&path, &buf).or(Err(Err::Reason("Could not write file".to_string())))?;
            Ok(Exp::Num(Number::from(n)))
        }
        _ => Err(Err::Reason("Expected second argument to be a list".to_string()))
    }
}

pub fn lisp_string(args: &[Exp]) -> Result<Exp, Err> {
    let args: Vec<String> = args.iter().map(|arg| match arg {
        Exp::Str(s) => format!("{}", s),
        exp => format!("{}", exp),
    }).collect();
    Ok(Exp::Str(args.join("")))
}

pub fn lisp_string_bytes(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let s = string(&args[0])?;
    let buf = s.as_bytes();
    Ok(Exp::List(buf.iter().map(|b| Exp::Num(Number::from(*b))).collect()))
}

pub fn lisp_bytes_string(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match &args[0] {
        Exp::List(list) => {
            let buf = bytes(list)?;
            let s = String::from_utf8(buf).or(Err(Err::Reason("Could not convert to valid UTF-8 string".to_string())))?;
            Ok(Exp::Str(s))
        }
        _ => Err(Err::Reason("Expected argument to be a list".to_string()))
    }
}

pub fn lisp_bytes_number(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match (&args[0], &args[1]) { // TODO: default type to "int" and make it optional
        (Exp::List(list), Exp::Str(kind)) => {
            let buf = bytes(list)?;
            ensure_length_eq!(buf, 8);
            match kind.as_str() { // TODO: bigint
                "int" => Ok(Exp::Num(Number::Int(i64::from_be_bytes(buf[0..8].try_into().unwrap())))),
                "float" => Ok(Exp::Num(Number::Float(f64::from_be_bytes(buf[0..8].try_into().unwrap())))),
                _ => Err(Err::Reason("Invalid number type".to_string())),
            }
        }
        _ => Err(Err::Reason("Expected args to be the number type and a list of bytes".to_string()))
    }
}

pub fn lisp_number_bytes(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let n = number(&args[0])?;
    Ok(Exp::List(n.to_be_bytes().iter().map(|b| Exp::Num(Number::from(*b))).collect()))
}

pub fn lisp_regex_find(args: &[Exp]) -> Result<Exp, Err> {
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
}

pub fn lisp_string_number(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let s = string(&args[0])?;
    let n = s.parse().or(Err(Err::Reason("Could not parse number".to_string())))?;
    Ok(Exp::Num(n))
}

pub fn lisp_type(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let exp = match args[0] {
        Exp::Primitive(_) => "function",
        Exp::Function(_)  => "function",
        Exp::Macro(_)     => "macro",
        Exp::List(_)      => "list",
        Exp::Bool(_)      => "boolean",
        Exp::Str(_)       => "string",
        Exp::Sym(_)       => "symbol",
        Exp::Num(_)       => "number",
    };
    Ok(Exp::Str(exp.to_string()))
}

pub fn lisp_number_type(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match args[0] {
        Exp::Num(Number::Int(_)) => Ok(Exp::Str("int".to_string())),
        Exp::Num(Number::BigInt(_)) => Ok(Exp::Str("bigint".to_string())),
        Exp::Num(Number::Float(_)) => Ok(Exp::Str("float".to_string())),
        _ => Err(Err::Reason("Expected argument to be a number".to_string()))
    }
}

pub fn lisp_parse(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    let s = string(&args[0])?;
    let (_, exp) = parse(&s)?;
    Ok(exp)
}

pub fn lisp_list(args: &[Exp]) -> Result<Exp, Err> {
    Ok(Exp::List(args.to_vec()))
}

pub fn lisp_unique(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    if let Exp::List(list) = &args[0] {
        let mut list = list.clone();
        list.dedup();
        Ok(Exp::List(list))
    } else {
        Err(Err::Reason("Expected argument to be a list".to_string()))
    }
}

pub fn lisp_sort(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    if let Exp::List(list) = &args[0] {
        let mut list = list.clone();
        list.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(Equal));
        Ok(Exp::List(list))
    } else {
        Err(Err::Reason("Expected argument to be a list".to_string()))
    }
}

pub fn lisp_contains(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    if let Exp::List(list) = &args[0] {
        Ok(Exp::Bool(list.contains(&args[1])))
    } else {
        Err(Err::Reason("Expected first argument to be a list".to_string()))
    }
}

pub fn lisp_nth(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    let i = usize::try_from(number(&args[1])?)?;
    match &args[0] {
        Exp::List(l) => {
            if let Some(e) = l.iter().nth(i) {
                Ok(e.clone())
            } else {
                Ok(Exp::List(Vec::new()))
            }
        }
        Exp::Str(s) => {
            if let Some(c) = s.chars().nth(i) {
                Ok(Exp::Str(c.to_string()))
            } else {
                Ok(Exp::Str("".to_string()))
            }
        }
        _ => Err(Err::Reason("Expected first argument to be a list or a string".to_string()))
    }
}

pub fn lisp_slice(args: &[Exp]) -> Result<Exp, Err> {
    let (a, b) = match args.len() {
        2 => (usize::try_from(number(&args[1])?)?, 1),
        3 => (usize::try_from(number(&args[1])?)?, usize::try_from(number(&args[2])?)?),
        _ => return Err(Err::Reason("Expected 2 or 3 args".to_string())),
    };
    match &args[0] {
        Exp::List(l) => {
            let l: Vec<Exp> = l.iter().cloned().skip(a).take(b).collect();
            Ok(Exp::List(l))
        }
        Exp::Str(s) => {
            let s: String = s.chars().skip(a).take(b).collect();
            Ok(Exp::Str(s))
        }
        _ => Err(Err::Reason("Expected first argument to be a list or a string".to_string()))
    }
}

pub fn lisp_chunks(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match (&args[0], &args[1]) {
        (Exp::List(list), Exp::Num(num)) => {
            let n = usize::try_from(num.clone())?;
            Ok(Exp::List(list.chunks(n).map(|a| Exp::List(a.to_vec())).collect()))
        }
        _ => Err(Err::Reason("Expected a list and a number".to_string()))
    }
}

pub fn lisp_split(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 2);
    match (&args[0], &args[1]) {
        (Exp::Str(string), Exp::Str(pattern)) => {
            let list = if pattern.is_empty() {
                // NOTE: "abc".split("") => ["", "b", "c", ""]
                string.chars().map(|s| Exp::Str(s.to_string())).collect()
            } else {
                string.split(pattern).map(|s| Exp::Str(s.to_string())).collect()
            };
            Ok(Exp::List(list))
        }
        _ => Err(Err::Reason("Expected a string and a pattern".to_string()))
    }
}

pub fn lisp_trim(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    if let Exp::Str(s) = &args[0] {
        Ok(Exp::Str(s.trim().to_string()))
    } else {
        Err(Err::Reason("Expected a string and a pattern".to_string()))
    }
}

pub fn lisp_length(args: &[Exp]) -> Result<Exp, Err> {
    ensure_length_eq!(args, 1);
    match &args[0] {
        Exp::List(list) => Ok(Exp::Num(Number::from(list.len()))),
        Exp::Str(string) => Ok(Exp::Num(Number::from(string.chars().count()))),
        _ => Err(Err::Reason("Expected a list or a string".to_string()))
    }
}

pub fn lisp_append(args: &[Exp]) -> Result<Exp, Err> {
    let mut res = vec![];
    for arg in args {
        if let Exp::List(list) = arg {
            res.extend_from_slice(list);
        } else {
            return Err(Err::Reason("Expected a list".to_string()))
        }
    }
    Ok(Exp::List(res))
}
