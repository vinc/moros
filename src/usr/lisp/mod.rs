mod env;
mod eval;
mod expand;
mod number;
mod parse;
mod primitive;

pub use env::Env;
pub use number::Number;

use env::default_env;
use eval::{eval, eval_variable_args};
use expand::expand;
use parse::parse;

use crate::api::console::Style;
use crate::api::fs;
use crate::api::process::ExitCode;
use crate::api::prompt::Prompt;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::cmp;
use core::convert::TryInto;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

// MOROS Lisp is a lisp-1 like Scheme and Clojure
//
// Eval & Env adapted from Risp
// Copyright 2019 Stepan Parunashvili
// https://github.com/stopachka/risp
//
// Parser rewritten from scratch using Nom
// https://github.com/geal/nom
//
// References:
//
// "Recursive Functions of Symic Expressions and Their Computation by Machine"
// by John McCarthy (1960)
//
// "The Roots of Lisp"
// by Paul Graham (2002)
//
// "Technical Issues of Separation in Function Cells and Value Cells"
// by Richard P. Gabriel (1982)

// Types

#[derive(Clone)]
pub enum Exp {
    Primitive(fn(&[Exp]) -> Result<Exp, Err>),
    Function(Box<Function>),
    Macro(Box<Function>),
    List(Vec<Exp>),
    Dict(BTreeMap<String, Exp>),
    Bool(bool),
    Num(Number),
    Str(String),
    Sym(String),
}

impl Exp {
    pub fn is_truthy(&self) -> bool {
        match self {
            Exp::Bool(b) => *b,
            Exp::List(l) => !l.is_empty(),
            _ => true,
        }
    }
}

impl PartialEq for Exp {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Exp::Function(a), Exp::Function(b)) => a == b,
            (Exp::Macro(a), Exp::Macro(b)) => a == b,
            (Exp::List(a), Exp::List(b)) => a == b,
            (Exp::Dict(a), Exp::Dict(b)) => a == b,
            (Exp::Bool(a), Exp::Bool(b)) => a == b,
            (Exp::Num(a), Exp::Num(b)) => a == b,
            (Exp::Str(a), Exp::Str(b)) => a == b,
            (Exp::Sym(a), Exp::Sym(b)) => a == b,
            _ => false,
        }
    }
}

impl PartialOrd for Exp {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        match (self, other) {
            (Exp::Function(a), Exp::Function(b)) => a.partial_cmp(b),
            (Exp::Macro(a), Exp::Macro(b)) => a.partial_cmp(b),
            (Exp::List(a), Exp::List(b)) => a.partial_cmp(b),
            (Exp::Dict(a), Exp::Dict(b)) => a.partial_cmp(b),
            (Exp::Bool(a), Exp::Bool(b)) => a.partial_cmp(b),
            (Exp::Num(a), Exp::Num(b)) => a.partial_cmp(b),
            (Exp::Str(a), Exp::Str(b)) => a.partial_cmp(b),
            (Exp::Sym(a), Exp::Sym(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            Exp::Primitive(_) => format!("(fun args)"),
            Exp::Function(f) => format!("(fun {})", f.params),
            Exp::Macro(m) => format!("(mac {})", m.params),
            Exp::Bool(a) => a.to_string(),
            Exp::Num(n) => n.to_string(),
            Exp::Sym(s) => s.clone(),
            Exp::Str(s) => {
                format!("{:?}", s).
                    replace("\\u{8}", "\\b").replace("\\u{1b}", "\\e")
            }
            Exp::List(list) => {
                let xs: Vec<_> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(" "))
            }
            Exp::Dict(dict) => {
                let mut xs: Vec<_> = dict.iter().map(|(k, v)|
                    format!("{} {}", k, v)
                ).collect();
                xs.insert(0, "dict".into());
                format!("({})", xs.join(" "))
            }
        };
        write!(f, "{}", out)
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Function {
    params: Exp,
    body: Exp,
    doc: Option<String>,
}

#[derive(Debug)]
pub enum Err {
    Reason(String),
}

lazy_static! {
    pub static ref FUNCTIONS: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

#[macro_export]
macro_rules! ensure_length_eq {
    ($list:expr, $count:expr) => {
        if $list.len() != $count {
            let plural = if $count != 1 { "s" } else { "" };
            return expected!("{} expression{}", $count, plural);
        }
    };
}

#[macro_export]
macro_rules! ensure_length_gt {
    ($list:expr, $count:expr) => {
        if $list.len() <= $count {
            let plural = if $count != 1 { "s" } else { "" };
            return expected!("more than {} expression{}", $count, plural);
        }
    };
}

#[macro_export]
macro_rules! ensure_string {
    ($exp:expr) => {
        match $exp {
            Exp::Str(_) => {}
            _ => return expected!("a string"),
        }
    };
}

#[macro_export]
macro_rules! ensure_list {
    ($exp:expr) => {
        match $exp {
            Exp::List(_) => {}
            _ => return expected!("a list"),
        }
    };
}

#[macro_export]
macro_rules! expected {
    ($($arg:tt)*) => ({
        use alloc::format;
        Err(Err::Reason(format!("Expected {}", format_args!($($arg)*))))
    });
}

#[macro_export]
macro_rules! could_not {
    ($($arg:tt)*) => ({
        use alloc::format;
        Err(Err::Reason(format!("Could not {}", format_args!($($arg)*))))
    });
}

pub fn bytes(args: &[Exp]) -> Result<Vec<u8>, Err> {
    args.iter().map(byte).collect()
}

pub fn strings(args: &[Exp]) -> Result<Vec<String>, Err> {
    args.iter().map(string).collect()
}

pub fn numbers(args: &[Exp]) -> Result<Vec<Number>, Err> {
    args.iter().map(number).collect()
}

pub fn string(exp: &Exp) -> Result<String, Err> {
    match exp {
        Exp::Str(s) => Ok(s.to_string()),
        _ => expected!("a string"),
    }
}

pub fn number(exp: &Exp) -> Result<Number, Err> {
    match exp {
        Exp::Num(num) => Ok(num.clone()),
        _ => expected!("a number"),
    }
}

pub fn float(exp: &Exp) -> Result<f64, Err> {
    match exp {
        Exp::Num(num) => Ok(num.into()),
        _ => expected!("a float"),
    }
}

pub fn byte(exp: &Exp) -> Result<u8, Err> {
    number(exp)?.try_into()
}

// REPL

fn parse_eval(
    input: &str,
    env: &mut Rc<RefCell<Env>>
) -> Result<(String, Exp), Err> {
    let (rest, exp) = parse(input)?;
    let exp = expand(&exp, env)?;
    let exp = eval(&exp, env)?;
    Ok((rest, exp))
}

fn exec(path: &str, env: &mut Rc<RefCell<Env>>) -> Result<(), Err> {
    let buf = fs::read_to_string(&path).or(could_not!("read file {:?}", path))?;
    let mut input = buf.clone();
    loop {
        match parse_eval(&input, env) {
            Ok((rest, _)) => {
                if rest.is_empty() {
                    break;
                }
                input = rest;
            }
            Err(Err::Reason(msg)) => {
                let i = buf.len() - input.trim_start().len();
                let row = buf[0..i].chars().filter(|c| *c == '\n').count();
                error!("{} at {}:{}", msg, path, row + 1);
                return could_not!("load file");
            }
        }
    }
    Ok(())
}

fn lisp_completer(line: &str) -> Vec<String> {
    let mut entries = Vec::new();
    if let Some(last_word) = line.split_whitespace().next_back() {
        if let Some(f) = last_word.strip_prefix('(') {
            for function in &*FUNCTIONS.lock() {
                if let Some(entry) = function.strip_prefix(f) {
                    entries.push(entry.into());
                }
            }
        }
    }
    entries
}

fn repl(env: &mut Rc<RefCell<Env>>) -> Result<(), ExitCode> {
    let csi_color = Style::color("teal");
    let csi_reset = Style::reset();
    let prompt_string = format!("{}>{} ", csi_color, csi_reset);

    println!("MOROS Lisp v0.9.0\n");

    let mut prompt = Prompt::new();
    let history_file = "~/.lisp-history";
    prompt.history.load(history_file);
    prompt.completion.set(&lisp_completer);

    while let Some(input) = prompt.input(&prompt_string) {
        if input == "(quit)" {
            break;
        }
        if input.is_empty() {
            println!();
            continue;
        }
        match parse_eval(&input, env) {
            Ok((_, exp)) => {
                println!("{}\n", exp);
            }
            Err(e) => match e {
                Err::Reason(msg) => error!("{}\n", msg),
            },
        }
        prompt.history.add(&input);
        prompt.history.save(history_file);
    }
    Ok(())
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let env = &mut default_env();

    // Store args in env
    let key = Exp::Sym("args".to_string());
    let list = Exp::List(if args.len() < 2 {
        vec![]
    } else {
        args[2..].iter().map(|arg| Exp::Str(arg.to_string())).collect()
    });
    let quote = Exp::List(vec![Exp::Sym("quote".to_string()), list]);
    if eval_variable_args(&[key, quote], env).is_err() {
        error!("Could not parse args");
        return Err(ExitCode::Failure);
    }

    if args.len() < 2 {
        let init = "/ini/lisp.lsp";
        if fs::exists(init) {
            exec(init, env).map_err(|_| ExitCode::Failure)?;
        }
        repl(env)
    } else {
        if args[1] == "-h" || args[1] == "--help" {
            return help();
        }
        exec(args[1], env).map_err(|_| ExitCode::Failure)
    }
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} lisp {}[<file> [<args>]]{}",
        csi_title, csi_reset, csi_option, csi_reset
    );
    Ok(())
}

#[test_case]
fn test_exp() {
    assert_eq!(Exp::Bool(true).is_truthy(), true);
    assert_eq!(Exp::Bool(false).is_truthy(), false);
    assert_eq!(Exp::Num(Number::Int(42)).is_truthy(), true);
    assert_eq!(Exp::List(vec![]).is_truthy(), false);
}

#[allow(unused_must_use)]
#[test_case]
fn test_lisp() {
    use core::f64::consts::PI;
    let env = &mut default_env();

    macro_rules! eval {
        ($e:expr) => {
            format!("{}", parse_eval($e, env).unwrap().1)
        };
    }

    // num
    assert_eq!(eval!("6"), "6");
    assert_eq!(eval!("16"), "16");
    assert_eq!(eval!("0x6"), "6");
    assert_eq!(eval!("0xf"), "15");
    assert_eq!(eval!("0x10"), "16");
    assert_eq!(eval!("1.5"), "1.5");
    assert_eq!(eval!("0xff"), "255");
    assert_eq!(eval!("0b0"), "0");
    assert_eq!(eval!("0b1"), "1");
    assert_eq!(eval!("0b10"), "2");
    assert_eq!(eval!("0b11"), "3");

    assert_eq!(eval!("-6"), "-6");
    assert_eq!(eval!("-16"), "-16");
    assert_eq!(eval!("-0x6"), "-6");
    assert_eq!(eval!("-0xF"), "-15");
    assert_eq!(eval!("-0x10"), "-16");
    assert_eq!(eval!("-1.5"), "-1.5");
    assert_eq!(eval!("-0xff"), "-255");
    assert_eq!(eval!("-0b11"), "-3");
    assert_eq!(eval!("123_456"), "123456");
    assert_eq!(eval!("0x123_456"), "1193046");
    assert_eq!(eval!("0.123_456"), "0.123456");

    // quote
    assert_eq!(eval!("(quote (1 2 3))"), "(1 2 3)");
    assert_eq!(eval!("'(1 2 3)"), "(1 2 3)");
    assert_eq!(eval!("(quote 1)"), "1");
    assert_eq!(eval!("'1"), "1");
    assert_eq!(eval!("(quote a)"), "a");
    assert_eq!(eval!("'a"), "a");
    assert_eq!(eval!("(quote '(a b c))"), "(quote (a b c))");

    // atom?
    assert_eq!(eval!("(atom? (quote a))"), "true");
    assert_eq!(eval!("(atom? (quote (1 2 3)))"), "false");
    assert_eq!(eval!("(atom? 1)"), "true");

    // eq?
    assert_eq!(eval!("(eq? (quote a) (quote a))"), "true");
    assert_eq!(eval!("(eq? (quote a) (quote b))"), "false");
    assert_eq!(eval!("(eq? (quote a) (quote ()))"), "false");
    assert_eq!(eval!("(eq? (quote ()) (quote ()))"), "true");
    assert_eq!(eval!("(eq? \"a\" \"a\")"), "true");
    assert_eq!(eval!("(eq? \"a\" \"b\")"), "false");
    assert_eq!(eval!("(eq? \"a\" 'b)"), "false");
    assert_eq!(eval!("(eq? 1 1)"), "true");
    assert_eq!(eval!("(eq? 1 1.0)"), "true");
    assert_eq!(eval!("(eq? 1.0 1.0)"), "true");
    assert_eq!(eval!("(eq? 1 2)"), "false");
    assert_eq!(eval!("(eq? (add 0.15 0.15) (add 0.1 0.2))"), "false");

    // cons
    assert_eq!(eval!("(cons (quote 1) (quote (2 3)))"), "(1 2 3)");
    assert_eq!(
        eval!("(cons (quote 1) (cons (quote 2) (cons (quote 3) (quote ()))))"),
        "(1 2 3)"
    );

    // case
    assert_eq!(
        eval!("(case 2 (1 \"one\") (2 \"two\") (3 \"three\"))"),
        "\"two\""
    );

    // cond
    assert_eq!(eval!("(cond ((lt? 2 4) 1))"), "1");
    assert_eq!(eval!("(cond ((gt? 2 4) 1))"), "()");
    assert_eq!(eval!("(cond ((lt? 2 4) 1) (true 2))"), "1");
    assert_eq!(eval!("(cond ((gt? 2 4) 1) (true 2))"), "2");

    // if
    assert_eq!(eval!("(if (lt? 2 4) 1)"), "1");
    assert_eq!(eval!("(if (gt? 2 4) 1)"), "()");
    assert_eq!(eval!("(if (lt? 2 4) 1 2)"), "1");
    assert_eq!(eval!("(if (gt? 2 4) 1 2)"), "2");
    assert_eq!(eval!("(if true 1 2)"), "1");
    assert_eq!(eval!("(if false 1 2)"), "2");
    assert_eq!(eval!("(if '() 1 2)"), "2");
    assert_eq!(eval!("(if 0 1 2)"), "1");
    assert_eq!(eval!("(if 42 1 2)"), "1");
    assert_eq!(eval!("(if \"\" 1 2)"), "1");

    // var
    eval!("(var a 2)");
    assert_eq!(eval!("(add a 1)"), "3");
    eval!("(var add-one (fun (b) (add b 1)))");
    assert_eq!(eval!("(add-one 2)"), "3");
    eval!("(var fibonacci (fun (n) \
        (if (lt? n 2) n (add (fibonacci (sub n 1)) (fibonacci (sub n 2))))))");
    assert_eq!(eval!("(fibonacci 6)"), "8");

    // var?
    assert_eq!(eval!("(var? a)"), "true");
    assert_eq!(eval!("(var? b)"), "false");

    // mut
    assert_eq!(eval!("(mut a 3)"), "3");
    assert_eq!(eval!("a"), "3");
    eval!("(var incr-a (fun () (mut a (add a 1))))");
    assert_eq!(eval!("a"), "3");
    assert_eq!(eval!("(incr-a)"), "4"); // Mutate var in outer scope
    assert_eq!(eval!("a"), "4");

    // while
    assert_eq!(
        eval!("(do (var i 0) (while (lt? i 5) (mut i (add i 1))) i)"),
        "5"
    );

    // function
    assert_eq!(eval!("((fun (a) (add 1 a)) 2)"), "3");
    assert_eq!(eval!("((fun (a) (mul a a)) 2)"), "4");
    assert_eq!(eval!("((fun (x) (cons x '(b c))) 'a)"), "(a b c)");
    assert_eq!(eval!("(fun () 42)"), "(fun ())");
    assert_eq!(eval!("((fun () 42))"), "42");

    // function definition shortcut
    eval!("(def (double x) (mul x 2))");
    assert_eq!(eval!("(double 2)"), "4");
    eval!("(def-fun (triple x) (mul x 3))");
    assert_eq!(eval!("(triple 2)"), "6");

    // add
    assert_eq!(eval!("(add)"), "0");
    assert_eq!(eval!("(add 2)"), "2");
    assert_eq!(eval!("(add 2 2)"), "4");
    assert_eq!(eval!("(add 2 3 4)"), "9");
    assert_eq!(eval!("(add 2 (add 3 4))"), "9");

    // sub
    assert_eq!(eval!("(sub 2)"), "-2");
    assert_eq!(eval!("(sub 2 1)"), "1");
    assert_eq!(eval!("(sub 1 2)"), "-1");
    assert_eq!(eval!("(sub 2 -1)"), "3");
    assert_eq!(eval!("(sub 8 4 2)"), "2");

    // mul
    assert_eq!(eval!("(mul)"), "1");
    assert_eq!(eval!("(mul 2)"), "2");
    assert_eq!(eval!("(mul 2 2)"), "4");
    assert_eq!(eval!("(mul 2 3 4)"), "24");
    assert_eq!(eval!("(mul 2 (mul 3 4))"), "24");

    // div
    assert_eq!(eval!("(div 4)"), "0");
    assert_eq!(eval!("(div 4.0)"), "0.25");
    assert_eq!(eval!("(div 4 2)"), "2");
    assert_eq!(eval!("(div 1 2)"), "0");
    assert_eq!(eval!("(div 1 2.0)"), "0.5");
    assert_eq!(eval!("(div 8 4 2)"), "1");

    // exp
    assert_eq!(eval!("(exp 2 4)"), "16");
    assert_eq!(eval!("(exp 2 4 2)"), "256"); // Left to right

    // rem
    assert_eq!(eval!("(rem 0 2)"), "0");
    assert_eq!(eval!("(rem 1 2)"), "1");
    assert_eq!(eval!("(rem 2 2)"), "0");
    assert_eq!(eval!("(rem 3 2)"), "1");
    assert_eq!(eval!("(rem -1 2)"), "-1");

    // cmp
    assert_eq!(eval!("(lt? 6 4)"), "false");
    assert_eq!(eval!("(gt? 6 4)"), "true");
    assert_eq!(eval!("(gt? 6 4 2)"), "true");
    assert_eq!(eval!("(gt? 6)"), "true");
    assert_eq!(eval!("(gt?)"), "true");
    assert_eq!(eval!("(gt? 6.0 4)"), "true");
    assert_eq!(eval!("(gte? 6 4 2)"), "true");
    assert_eq!(eval!("(gte? 6 4 4)"), "true");
    assert_eq!(eval!("(gte? 4 4 2)"), "true");
    assert_eq!(eval!("(gte? 4 4 4)"), "true");
    assert_eq!(eval!("(gte? 4 4 4.0)"), "true");
    assert_eq!(eval!("(gte? 2 4 4.0)"), "false");
    assert_eq!(eval!("(num/eq? 6 4)"), "false");
    assert_eq!(eval!("(num/eq? 6 6)"), "true");
    assert_eq!(eval!("(num/eq? 6 6.0)"), "false");
    assert_eq!(eval!("(num/eq? (add 0.15 0.15) (add 0.1 0.2))"), "false");

    // bit/and
    assert_eq!(eval!("(bit/and 1 2)"), "0");
    assert_eq!(eval!("(bit/and 1 3)"), "1");
    assert_eq!(eval!("(bit/and 1 2 3)"), "0");

    // bit/xor
    assert_eq!(eval!("(bit/xor 1 2)"), "3");
    assert_eq!(eval!("(bit/xor 1 3)"), "2");
    assert_eq!(eval!("(bit/xor 1 2 3)"), "0");

    // bit/or
    assert_eq!(eval!("(bit/or 1 2)"), "3");
    assert_eq!(eval!("(bit/or 1 3)"), "3");
    assert_eq!(eval!("(bit/or 1 2 3)"), "3");

    // bit/shl
    assert_eq!(eval!("(bit/shl 2 10)"), "2048");
    assert_eq!(eval!("(bit/shl 2.0 10)"), "NaN");

    // num/int
    assert_eq!(eval!("(num/int 2)"), "2");
    assert_eq!(eval!("(num/int 2.0)"), "2");
    assert_eq!(eval!("(num/int 2.4)"), "2");
    assert_eq!(eval!("(num/int 2.6)"), "2");
    assert_eq!(eval!("(num/int -2.6)"), "-2");

    // number
    assert_eq!(eval!("(bin->num (num->bin 42) \"int\")"), "42");
    assert_eq!(eval!("(bin->num (num->bin 42.0) \"float\")"), "42.0");

    // str
    assert_eq!(eval!("(parse \"9.75\")"), "9.75");
    assert_eq!(eval!("(str \"a\" \"b\" \"c\")"), "\"abc\"");
    assert_eq!(eval!("(str \"a\" \"\")"), "\"a\"");
    assert_eq!(eval!("(str \"foo \" 3)"), "\"foo 3\"");
    assert_eq!(eval!("(eq? \"foo\" \"foo\")"), "true");
    assert_eq!(eval!("(eq? \"foo\" \"bar\")"), "false");
    assert_eq!(eval!("(str/trim \"abc\n\")"), "\"abc\"");
    assert_eq!(
        eval!("(str/split \"a\nb\nc\" \"\n\")"),
        "(\"a\" \"b\" \"c\")"
    );

    // trigo
    assert_eq!(eval!("(acos (cos pi))"), PI.to_string());
    assert_eq!(eval!("(acos 0)"), (PI / 2.0).to_string());
    assert_eq!(eval!("(asin 1)"), (PI / 2.0).to_string());
    assert_eq!(eval!("(atan 0)"), "0.0");
    assert_eq!(eval!("(cos pi)"), "-1.0");
    assert_eq!(eval!("(sin (div pi 2))"), "1.0");
    assert_eq!(eval!("(tan 0)"), "0.0");

    // list
    assert_eq!(eval!("(list)"), "()");
    assert_eq!(eval!("(list 1)"), "(1)");
    assert_eq!(eval!("(list 1 2)"), "(1 2)");
    assert_eq!(eval!("(list 1 2 (add 1 2))"), "(1 2 3)");

    // bigint
    assert_eq!(
        eval!("9223372036854775807"),
        "9223372036854775807" // -> int
    );
    assert_eq!(
        eval!("9223372036854775808"),
        "9223372036854775808" // -> bigint
    );
    assert_eq!(
        eval!("0x7fffffffffffffff"),
        "9223372036854775807" // -> int
    );
    assert_eq!(
        eval!("0x8000000000000000"),
        "9223372036854775808" // -> bigint
    );
    assert_eq!(
        eval!("0x800000000000000f"),
        "9223372036854775823" // -> bigint
    );
    assert_eq!(
        eval!("(add 9223372036854775807 0)"),
        "9223372036854775807" // -> int
    );
    assert_eq!(
        eval!("(sub 9223372036854775808 1)"),
        "9223372036854775807" // -> bigint
    );
    assert_eq!(
        eval!("(add 9223372036854775807 1)"),
        "9223372036854775808" // -> bigint
    );
    assert_eq!(
        eval!("(add 9223372036854775807 1.0)"),
        "9223372036854776000.0" // -> float
    );
    assert_eq!(
        eval!("(add 9223372036854775807 10)"),
        "9223372036854775817" // -> bigint
    );
    assert_eq!(
        eval!("(mul 9223372036854775807 10)"),
        "92233720368547758070" // -> bigint
    );

    assert_eq!(
        eval!("(exp 2 16)"),
        "65536" // -> int
    );
    assert_eq!(
        eval!("(exp 2 128)"),
        "340282366920938463463374607431768211456" // -> bigint
    );
    assert_eq!(
        eval!("(exp 2.0 128)"),
        "340282366920938500000000000000000000000.0" // -> float
    );

    assert_eq!(eval!("(num/type 9223372036854775807)"), "\"int\"");
    assert_eq!(eval!("(num/type 9223372036854775808)"), "\"bigint\"");
    assert_eq!(eval!("(num/type 9223372036854776000.0)"), "\"float\"");

    // quasiquote
    eval!("(var x 'a)");
    assert_eq!(eval!("`(x ,x y)"), "(x a y)");
    assert_eq!(eval!("`(x ,x y ,(add 1 2))"), "(x a y 3)");
    assert_eq!(eval!("`(list ,(add 1 2) 4)"), "(list 3 4)");

    // unquote-splice
    eval!("(var x '(1 2 3))");
    assert_eq!(eval!("`(add ,x)"), "(add (1 2 3))");
    assert_eq!(eval!("`(add ,@x)"), "(add 1 2 3)");

    // splice
    assert_eq!(eval!("((fun (a @b) a) 1 2 3)"), "1");
    assert_eq!(eval!("((fun (a @b) b) 1 2 3)"), "(2 3)");

    // mac
    eval!("(var foo 42)");
    eval!("(var mut-10 (mac (x) `(mut ,x 10)))");
    eval!("(mut-10 foo)");
    assert_eq!(eval!("foo"), "10");

    // args
    eval!("(var list* (fun args (concat args '())))");
    assert_eq!(eval!("(list* 1 2 3)"), "(1 2 3)");

    // comments
    assert_eq!(eval!("# comment"), "()");
    assert_eq!(eval!("# comment\n# comment"), "()");
    assert_eq!(eval!("(add 1 2 3) # comment"), "6");
    assert_eq!(eval!("(add 1 2 3) # comment\n# comment"), "6");

    // list
    assert_eq!(eval!("(list 1 2 3)"), "(1 2 3)");

    // dict
    assert_eq!(
        eval!("(dict \"a\" 1 \"b\" 2 \"c\" 3)"),
        "(dict \"a\" 1 \"b\" 2 \"c\" 3)"
    );

    // dict/pairs
    assert_eq!(
        eval!("(dict/pairs (dict \"a\" 1 \"b\" 2 \"c\" 3))"),
        "((\"a\" 1) (\"b\" 2) (\"c\" 3))"
    );

    // len
    assert_eq!(eval!("(len (list))"), "0");
    assert_eq!(eval!("(len (dict))"), "0");
    assert_eq!(eval!("(len (list 1 2 3))"), "3");
    assert_eq!(eval!("(len (dict 1 1 2 2 3 3))"), "3");

    // get
    assert_eq!(eval!("(get \"Hello\" 0)"), "\"H\"");
    assert_eq!(eval!("(get \"Hello\" 6)"), "\"\"");
    assert_eq!(eval!("(get (list 1 2 3) 0)"), "1");
    assert_eq!(eval!("(get (list 1 2 3) 3)"), "()");
    assert_eq!(eval!("(get (dict \"a\" 1 \"b\" 2 \"c\" 3) \"a\")"), "1");
    assert_eq!(eval!("(get (dict \"a\" 1 \"b\" 2 \"c\" 3) \"d\")"), "()");

    // put
    assert_eq!(
        eval!("(put (dict \"a\" 1 \"b\" 2) \"c\" 3)"),
        "(dict \"a\" 1 \"b\" 2 \"c\" 3)"
    );
    assert_eq!(eval!("(put (list 1 3) 1 2)"), "(1 2 3)");
    assert_eq!(eval!("(put \"Heo\" 2 \"ll\")"), "\"Hello\"");

    // expand
    assert_eq!(eval!("(expand ())"), "()");
    assert_eq!(eval!("(expand '())"), "(quote ())");
    assert_eq!(
        eval!("(expand (def (double x) (mul x x)))"),
        "(var double (fun (x) (mul x x)))"
    );

    // eval
    assert_eq!(eval!("(eval 42)"), "42");
    assert_eq!(eval!("(eval \"hello\")"), "\"hello\"");
    assert_eq!(eval!("(eval (dict \"a\" 1))"), "(dict \"a\" 1)");

    // apply
    assert_eq!(eval!("(apply add '(1 2 3))"), "6");
    assert_eq!(eval!("(apply add 1 2 '(3))"), "6");
    assert_eq!(eval!("(apply add 1 2 '())"), "3");
    assert_eq!(eval!("(apply add '())"), "0");
    assert_eq!(eval!("(apply (fun (x) x) (list '(1 2))))"), "(1 2)");

    // fold
    assert_eq!(eval!("(fold sub 0 '(1 2 3))"), "-6");
    assert_eq!(eval!("(fold sub 0 '(1 2))"), "-3");
    assert_eq!(eval!("(fold sub 0 '(1))"), "-1");
    assert_eq!(eval!("(fold sub 0 '())"), "0");
}
