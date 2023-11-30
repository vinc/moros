mod env;
mod eval;
mod expand;
mod number;
mod parse;
mod primitive;

pub use number::Number;
pub use env::Env;

use env::default_env;
use eval::{eval, eval_variable_args};
use expand::expand;
use parse::parse;

use crate::api;
use crate::api::console::Style;
use crate::api::process::ExitCode;
use crate::api::prompt::Prompt;

use alloc::boxed::Box;
use alloc::format;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::vec;
use core::cell::RefCell;
use core::convert::TryInto;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

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
// MOROS Lisp is a lisp-1 like Scheme and Clojure
// See "Technical Issues of Separation in Function Cells and Value Cells" by Richard P. Gabriel (1982)

// Types

#[derive(Clone)]
pub enum Exp {
    Primitive(fn(&[Exp]) -> Result<Exp, Err>),
    Function(Box<Function>),
    Macro(Box<Function>),
    List(Vec<Exp>),
    Bool(bool),
    Num(Number),
    Str(String),
    Sym(String),
}

impl PartialEq for Exp {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Exp::Function(a), Exp::Function(b)) => a == b,
            (Exp::Macro(a),    Exp::Macro(b))    => a == b,
            (Exp::List(a),     Exp::List(b))     => a == b,
            (Exp::Bool(a),     Exp::Bool(b))     => a == b,
            (Exp::Num(a),      Exp::Num(b))      => a == b,
            (Exp::Str(a),      Exp::Str(b))      => a == b,
            (Exp::Sym(a),      Exp::Sym(b))      => a == b,
            _ => false,
        }
    }
}

use core::cmp::Ordering;
impl PartialOrd for Exp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Exp::Function(a), Exp::Function(b)) => a.partial_cmp(b),
            (Exp::Macro(a),    Exp::Macro(b))    => a.partial_cmp(b),
            (Exp::List(a),     Exp::List(b))     => a.partial_cmp(b),
            (Exp::Bool(a),     Exp::Bool(b))     => a.partial_cmp(b),
            (Exp::Num(a),      Exp::Num(b))      => a.partial_cmp(b),
            (Exp::Str(a),      Exp::Str(b))      => a.partial_cmp(b),
            (Exp::Sym(a),      Exp::Sym(b))      => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            Exp::Primitive(_) => format!("(function args)"),
            Exp::Function(f)  => format!("(function {})", f.params),
            Exp::Macro(m)     => format!("(macro {})", m.params),
            Exp::Bool(a)      => a.to_string(),
            Exp::Num(n)       => n.to_string(),
            Exp::Sym(s)       => s.clone(),
            Exp::Str(s)       => format!("{:?}", s),
            Exp::List(list)   => {
                let xs: Vec<String> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(" "))
            },
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
            Exp::Str(_) => {},
            _ => return expected!("a string"),
        }
    };
}

#[macro_export]
macro_rules! ensure_list {
    ($exp:expr) => {
        match $exp {
            Exp::List(_) => {},
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

fn parse_eval(input: &str, env: &mut Rc<RefCell<Env>>) -> Result<(String, Exp), Err> {
    let (rest, exp) = parse(input)?;
    let exp = expand(&exp, env)?;
    let exp = eval(&exp, env)?;
    Ok((rest, exp))
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
    let csi_color = Style::color("Cyan");
    let csi_reset = Style::reset();
    let prompt_string = format!("{}>{} ", csi_color, csi_reset);

    println!("MOROS Lisp v0.6.0\n");

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
        repl(env)
    } else {
        if args[1] == "-h" || args[1] == "--help" {
            return help();
        }
        let path = args[1];
        if let Ok(mut input) = api::fs::read_to_string(path) {
            loop {
                match parse_eval(&input, env) {
                    Ok((rest, _)) => {
                        if rest.is_empty() {
                            break;
                        }
                        input = rest;
                    }
                    Err(Err::Reason(msg)) => {
                        error!("{}", msg);
                        return Err(ExitCode::Failure);
                    }
                }
            }
            Ok(())
        } else {
            error!("Could not read file '{}'", path);
            Err(ExitCode::Failure)
        }
    }
}

fn help() -> Result<(), ExitCode> {
    let csi_option = Style::color("LightCyan");
    let csi_title = Style::color("Yellow");
    let csi_reset = Style::reset();
    println!("{}Usage:{} lisp {}[<file> [<args>]]{}", csi_title, csi_reset, csi_option, csi_reset);
    Ok(())
}

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

    // equal?
    assert_eq!(eval!("(equal? (quote a) (quote a))"), "true");
    assert_eq!(eval!("(equal? (quote a) (quote b))"), "false");
    assert_eq!(eval!("(equal? (quote a) (quote ()))"), "false");
    assert_eq!(eval!("(equal? (quote ()) (quote ()))"), "true");
    assert_eq!(eval!("(equal? \"a\" \"a\")"), "true");
    assert_eq!(eval!("(equal? \"a\" \"b\")"), "false");
    assert_eq!(eval!("(equal? \"a\" 'b)"), "false");
    assert_eq!(eval!("(equal? 1 1)"), "true");
    assert_eq!(eval!("(equal? 1 2)"), "false");
    assert_eq!(eval!("(equal? 1 1.0)"), "false");
    assert_eq!(eval!("(equal? 1.0 1.0)"), "true");

    // head
    assert_eq!(eval!("(head (quote (1)))"), "1");
    assert_eq!(eval!("(head (quote (1 2 3)))"), "1");

    // tail
    assert_eq!(eval!("(tail (quote (1)))"), "()");
    assert_eq!(eval!("(tail (quote (1 2 3)))"), "(2 3)");

    // cons
    assert_eq!(eval!("(cons (quote 1) (quote (2 3)))"), "(1 2 3)");
    assert_eq!(eval!("(cons (quote 1) (cons (quote 2) (cons (quote 3) (quote ()))))"), "(1 2 3)");

    // cond
    assert_eq!(eval!("(cond ((< 2 4) 1))"), "1");
    assert_eq!(eval!("(cond ((> 2 4) 1))"), "()");
    assert_eq!(eval!("(cond ((< 2 4) 1) (true 2))"), "1");
    assert_eq!(eval!("(cond ((> 2 4) 1) (true 2))"), "2");

    // if
    assert_eq!(eval!("(if (< 2 4) 1)"), "1");
    assert_eq!(eval!("(if (> 2 4) 1)"), "()");
    assert_eq!(eval!("(if (< 2 4) 1 2)"), "1");
    assert_eq!(eval!("(if (> 2 4) 1 2)"), "2");

    // while
    assert_eq!(eval!("(do (variable i 0) (while (< i 5) (set i (+ i 1))) i)"), "5");

    // variable
    eval!("(variable a 2)");
    assert_eq!(eval!("(+ a 1)"), "3");
    eval!("(variable add-one (function (b) (+ b 1)))");
    assert_eq!(eval!("(add-one 2)"), "3");
    eval!("(variable fibonacci (function (n) (if (< n 2) n (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))");
    assert_eq!(eval!("(fibonacci 6)"), "8");

    // function
    assert_eq!(eval!("((function (a) (+ 1 a)) 2)"), "3");
    assert_eq!(eval!("((function (a) (* a a)) 2)"), "4");
    assert_eq!(eval!("((function (x) (cons x '(b c))) 'a)"), "(a b c)");

    // function definition shortcut
    eval!("(define (double x) (* x 2))");
    assert_eq!(eval!("(double 2)"), "4");
    eval!("(define-function (triple x) (* x 3))");
    assert_eq!(eval!("(triple 2)"), "6");

    // addition
    assert_eq!(eval!("(+)"), "0");
    assert_eq!(eval!("(+ 2)"), "2");
    assert_eq!(eval!("(+ 2 2)"), "4");
    assert_eq!(eval!("(+ 2 3 4)"), "9");
    assert_eq!(eval!("(+ 2 (+ 3 4))"), "9");

    // subtraction
    assert_eq!(eval!("(- 2)"), "-2");
    assert_eq!(eval!("(- 2 1)"), "1");
    assert_eq!(eval!("(- 1 2)"), "-1");
    assert_eq!(eval!("(- 2 -1)"), "3");
    assert_eq!(eval!("(- 8 4 2)"), "2");

    // multiplication
    assert_eq!(eval!("(*)"), "1");
    assert_eq!(eval!("(* 2)"), "2");
    assert_eq!(eval!("(* 2 2)"), "4");
    assert_eq!(eval!("(* 2 3 4)"), "24");
    assert_eq!(eval!("(* 2 (* 3 4))"), "24");

    // division
    assert_eq!(eval!("(/ 4)"), "0");
    assert_eq!(eval!("(/ 4.0)"), "0.25");
    assert_eq!(eval!("(/ 4 2)"), "2");
    assert_eq!(eval!("(/ 1 2)"), "0");
    assert_eq!(eval!("(/ 1 2.0)"), "0.5");
    assert_eq!(eval!("(/ 8 4 2)"), "1");

    // exponential
    assert_eq!(eval!("(^ 2 4)"), "16");
    assert_eq!(eval!("(^ 2 4 2)"), "256"); // Left to right

    // remainder
    assert_eq!(eval!("(rem 0 2)"), "0");
    assert_eq!(eval!("(rem 1 2)"), "1");
    assert_eq!(eval!("(rem 2 2)"), "0");
    assert_eq!(eval!("(rem 3 2)"), "1");
    assert_eq!(eval!("(rem -1 2)"), "-1");

    // comparisons
    assert_eq!(eval!("(< 6 4)"), "false");
    assert_eq!(eval!("(> 6 4)"), "true");
    assert_eq!(eval!("(> 6 4 2)"), "true");
    assert_eq!(eval!("(> 6)"), "true");
    assert_eq!(eval!("(>)"), "true");
    assert_eq!(eval!("(= 6 4)"), "false");
    assert_eq!(eval!("(= 6 6)"), "true");
    assert_eq!(eval!("(= (+ 0.15 0.15) (+ 0.1 0.2))"), "false"); // FIXME?

    // number
    assert_eq!(eval!("(binary->number (number->binary 42) \"int\")"), "42");
    assert_eq!(eval!("(binary->number (number->binary 42.0) \"float\")"), "42.0");

    // string
    assert_eq!(eval!("(parse \"9.75\")"), "9.75");
    assert_eq!(eval!("(string \"a\" \"b\" \"c\")"), "\"abc\"");
    assert_eq!(eval!("(string \"a\" \"\")"), "\"a\"");
    assert_eq!(eval!("(string \"foo \" 3)"), "\"foo 3\"");
    assert_eq!(eval!("(equal? \"foo\" \"foo\")"), "true");
    assert_eq!(eval!("(equal? \"foo\" \"bar\")"), "false");
    assert_eq!(eval!("(string.trim \"abc\n\")"), "\"abc\"");
    assert_eq!(eval!("(string.split \"a\nb\nc\" \"\n\")"), "(\"a\" \"b\" \"c\")");

    // apply
    assert_eq!(eval!("(apply + '(1 2 3))"), "6");
    assert_eq!(eval!("(apply + 1 '(2 3))"), "6");
    assert_eq!(eval!("(apply + 1 2 '(3))"), "6");
    assert_eq!(eval!("(apply + 1 2 3 '())"), "6");

    // trigo
    assert_eq!(eval!("(acos (cos pi))"), PI.to_string());
    assert_eq!(eval!("(acos 0)"), (PI / 2.0).to_string());
    assert_eq!(eval!("(asin 1)"), (PI / 2.0).to_string());
    assert_eq!(eval!("(atan 0)"), "0.0");
    assert_eq!(eval!("(cos pi)"), "-1.0");
    assert_eq!(eval!("(sin (/ pi 2))"), "1.0");
    assert_eq!(eval!("(tan 0)"), "0.0");

    // list
    assert_eq!(eval!("(list)"), "()");
    assert_eq!(eval!("(list 1)"), "(1)");
    assert_eq!(eval!("(list 1 2)"), "(1 2)");
    assert_eq!(eval!("(list 1 2 (+ 1 2))"), "(1 2 3)");

    // bigint
    assert_eq!(eval!("9223372036854775807"),          "9223372036854775807");   // -> int
    assert_eq!(eval!("9223372036854775808"),          "9223372036854775808");   // -> bigint
    assert_eq!(eval!("0x7fffffffffffffff"),           "9223372036854775807");   // -> int
    assert_eq!(eval!("0x8000000000000000"),           "9223372036854775808");   // -> bigint
    assert_eq!(eval!("0x800000000000000f"),           "9223372036854775823");   // -> bigint
    assert_eq!(eval!("(+ 9223372036854775807 0)"),    "9223372036854775807");   // -> int
    assert_eq!(eval!("(- 9223372036854775808 1)"),    "9223372036854775807");   // -> bigint
    assert_eq!(eval!("(+ 9223372036854775807 1)"),    "9223372036854775808");   // -> bigint
    assert_eq!(eval!("(+ 9223372036854775807 1.0)"),  "9223372036854776000.0"); // -> float
    assert_eq!(eval!("(+ 9223372036854775807 10)"),   "9223372036854775817");   // -> bigint
    assert_eq!(eval!("(* 9223372036854775807 10)"),  "92233720368547758070");   // -> bigint

    assert_eq!(eval!("(^ 2 16)"),                                      "65536");   // -> int
    assert_eq!(eval!("(^ 2 128)"),   "340282366920938463463374607431768211456");   // -> bigint
    assert_eq!(eval!("(^ 2.0 128)"), "340282366920938500000000000000000000000.0"); // -> float

    assert_eq!(eval!("(number.type 9223372036854775807)"),   "\"int\"");
    assert_eq!(eval!("(number.type 9223372036854775808)"),   "\"bigint\"");
    assert_eq!(eval!("(number.type 9223372036854776000.0)"), "\"float\"");

    // quasiquote
    eval!("(variable x 'a)");
    assert_eq!(eval!("`(x ,x y)"), "(x a y)");
    assert_eq!(eval!("`(x ,x y ,(+ 1 2))"), "(x a y 3)");
    assert_eq!(eval!("`(list ,(+ 1 2) 4)"), "(list 3 4)");

    // unquote-splice
    eval!("(variable x '(1 2 3))");
    assert_eq!(eval!("`(+ ,x)"), "(+ (1 2 3))");
    assert_eq!(eval!("`(+ ,@x)"), "(+ 1 2 3)");

    // splice
    assert_eq!(eval!("((function (a @b) a) 1 2 3)"), "1");
    assert_eq!(eval!("((function (a @b) b) 1 2 3)"), "(2 3)");

    // macro
    eval!("(variable foo 42)");
    eval!("(variable set-10 (macro (x) `(set ,x 10)))");
    eval!("(set-10 foo)");
    assert_eq!(eval!("foo"), "10");

    // args
    eval!("(variable list* (function args (concat args '())))");
    assert_eq!(eval!("(list* 1 2 3)"), "(1 2 3)");

    // comments
    assert_eq!(eval!("# comment"), "()");
    assert_eq!(eval!("# comment\n# comment"), "()");
    assert_eq!(eval!("(+ 1 2 3) # comment"), "6");
    assert_eq!(eval!("(+ 1 2 3) # comment\n# comment"), "6");
}
