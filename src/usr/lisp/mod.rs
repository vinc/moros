mod env;
mod eval;
mod expand;
mod number;
mod parse;

pub use number::Number;
pub use env::Env;

use env::default_env;
use eval::{eval, eval_define_args};
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

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            Exp::Primitive(_) => "<function>".to_string(),
            Exp::Function(_)  => "<function>".to_string(),
            Exp::Macro(_)     => "<macro>".to_string(),
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

#[derive(Clone, PartialEq)]
pub struct Function {
    params: Exp,
    body: Exp,
}

#[derive(Debug)]
pub enum Err {
    Reason(String),
}

lazy_static! {
    pub static ref FORMS: Mutex<Vec<String>> = Mutex::new(Vec::new());
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

pub fn list_of_numbers(args: &[Exp]) -> Result<Vec<Number>, Err> {
    args.iter().map(number).collect()
}

pub fn list_of_bytes(args: &[Exp]) -> Result<Vec<u8>, Err> {
    args.iter().map(byte).collect()
}

pub fn string(exp: &Exp) -> Result<String, Err> {
    match exp {
        Exp::Str(s) => Ok(s.to_string()),
        _ => Err(Err::Reason("Expected a string".to_string())),
    }
}

pub fn number(exp: &Exp) -> Result<Number, Err> {
    match exp {
        Exp::Num(num) => Ok(num.clone()),
        _ => Err(Err::Reason("Expected a number".to_string())),
    }
}

pub fn float(exp: &Exp) -> Result<f64, Err> {
    match exp {
        Exp::Num(num) => Ok(num.into()),
        _ => Err(Err::Reason("Expected a float".to_string())),
    }
}

pub fn byte(exp: &Exp) -> Result<u8, Err> {
    number(exp)?.try_into()
}

// REPL

fn parse_eval(exp: &str, env: &mut Rc<RefCell<Env>>) -> Result<Exp, Err> {
    let (_, exp) = parse(exp)?;
    let exp = expand(&exp, env)?;
    let exp = eval(&exp, env)?;
    Ok(exp)
}

fn strip_comments(s: &str) -> String {
    // FIXME: This doesn't handle `#` inside a string
    s.split('#').next().unwrap().into()
}

fn lisp_completer(line: &str) -> Vec<String> {
    let mut entries = Vec::new();
    if let Some(last_word) = line.split_whitespace().next_back() {
        if let Some(f) = last_word.strip_prefix('(') {
            for form in &*FORMS.lock() {
                if let Some(entry) = form.strip_prefix(f) {
                    entries.push(entry.into());
                }
            }
        }
    }
    entries
}

fn repl(env: &mut Rc<RefCell<Env>>) -> Result<(), ExitCode> {
    let csi_color = Style::color("Cyan");
    let csi_error = Style::color("LightRed");
    let csi_reset = Style::reset();
    let prompt_string = format!("{}>{} ", csi_color, csi_reset);

    println!("MOROS Lisp v0.4.0\n");

    let mut prompt = Prompt::new();
    let history_file = "~/.lisp-history";
    prompt.history.load(history_file);
    prompt.completion.set(&lisp_completer);

    while let Some(line) = prompt.input(&prompt_string) {
        if line == "(quit)" {
            break;
        }
        if line.is_empty() {
            println!();
            continue;
        }
        match parse_eval(&line, env) {
            Ok(res) => {
                println!("{}\n", res);
            }
            Err(e) => match e {
                Err::Reason(msg) => println!("{}Error:{} {}\n", csi_error, csi_reset, msg),
            },
        }
        prompt.history.add(&line);
        prompt.history.save(history_file);
    }
    Ok(())
}

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let line_color = Style::color("Yellow");
    let error_color = Style::color("LightRed");
    let reset = Style::reset();

    let env = &mut default_env();

    // Store args in env
    let key = Exp::Sym("args".to_string());
    let list = Exp::List(if args.len() < 2 {
        vec![]
    } else {
        args[2..].iter().map(|arg| Exp::Str(arg.to_string())).collect()
    });
    let quote = Exp::List(vec![Exp::Sym("quote".to_string()), list]);
    if eval_define_args(&[key, quote], env).is_err() {
        error!("Could not parse args");
        return Err(ExitCode::Failure);
    }

    if args.len() < 2 {
        repl(env)
    } else {
        let pathname = args[1];
        if let Ok(code) = api::fs::read_to_string(pathname) {
            let mut block = String::new();
            let mut opened = 0;
            let mut closed = 0;
            for (i, line) in code.split('\n').enumerate() {
                let line = strip_comments(line);
                if !line.is_empty() {
                    opened += line.matches('(').count();
                    closed += line.matches(')').count();
                    block.push_str(&line);
                    if closed >= opened {
                        if let Err(e) = parse_eval(&block, env) {
                            match e {
                                Err::Reason(msg) => {
                                    eprintln!("{}Error:{} {}", error_color, reset, msg);
                                    eprintln!();
                                    eprintln!("  {}{}:{} {}", line_color, i, reset, line);
                                    return Err(ExitCode::Failure);
                                }
                            }
                        }
                        block.clear();
                        opened = 0;
                        closed = 0;
                    }
                }
            }
            Ok(())
        } else {
            error!("File not found '{}'", pathname);
            Err(ExitCode::Failure)
        }
    }
}

#[test_case]
fn test_lisp() {
    use core::f64::consts::PI;
    let env = &mut default_env();

    macro_rules! eval {
        ($e:expr) => {
            format!("{}", parse_eval($e, env).unwrap())
        };
    }

    // quote
    assert_eq!(eval!("(quote (1 2 3))"), "(1 2 3)");
    assert_eq!(eval!("'(1 2 3)"), "(1 2 3)");
    assert_eq!(eval!("(quote 1)"), "1");
    assert_eq!(eval!("'1"), "1");
    assert_eq!(eval!("(quote a)"), "a");
    assert_eq!(eval!("'a"), "a");
    assert_eq!(eval!("(quote '(a b c))"), "(quote (a b c))");

    // atom
    assert_eq!(eval!("(atom (quote a))"), "true");
    assert_eq!(eval!("(atom (quote (1 2 3)))"), "false");
    assert_eq!(eval!("(atom 1)"), "true");

    // eq
    assert_eq!(eval!("(eq (quote a) (quote a))"), "true");
    assert_eq!(eval!("(eq (quote a) (quote b))"), "false");
    assert_eq!(eval!("(eq (quote a) (quote ()))"), "false");
    assert_eq!(eval!("(eq (quote ()) (quote ()))"), "true");
    assert_eq!(eval!("(eq \"a\" \"a\")"), "true");
    assert_eq!(eval!("(eq \"a\" \"b\")"), "false");
    assert_eq!(eval!("(eq \"a\" 'b)"), "false");
    assert_eq!(eval!("(eq 1 1)"), "true");
    assert_eq!(eval!("(eq 1 2)"), "false");
    assert_eq!(eval!("(eq 1 1.0)"), "false");
    assert_eq!(eval!("(eq 1.0 1.0)"), "true");

    // car
    assert_eq!(eval!("(car (quote (1)))"), "1");
    assert_eq!(eval!("(car (quote (1 2 3)))"), "1");

    // cdr
    assert_eq!(eval!("(cdr (quote (1)))"), "()");
    assert_eq!(eval!("(cdr (quote (1 2 3)))"), "(2 3)");

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
    assert_eq!(eval!("(do (define i 0) (while (< i 5) (set i (+ i 1))) i)"), "5");

    // define
    eval!("(define a 2)");
    assert_eq!(eval!("(+ a 1)"), "3");
    eval!("(define add-one (function (b) (+ b 1)))");
    assert_eq!(eval!("(add-one 2)"), "3");
    eval!("(define fibonacci (function (n) (if (< n 2) n (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))");
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

    // modulo
    assert_eq!(eval!("(% 3 2)"), "1");

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
    assert_eq!(eval!("(bytes->number (number->bytes 42) \"int\")"), "42");
    assert_eq!(eval!("(bytes->number (number->bytes 42.0) \"float\")"), "42.0");

    // string
    assert_eq!(eval!("(parse \"9.75\")"), "9.75");
    assert_eq!(eval!("(string \"a\" \"b\" \"c\")"), "\"abc\"");
    assert_eq!(eval!("(string \"a\" \"\")"), "\"a\"");
    assert_eq!(eval!("(string \"foo \" 3)"), "\"foo 3\"");
    assert_eq!(eval!("(eq \"foo\" \"foo\")"), "true");
    assert_eq!(eval!("(eq \"foo\" \"bar\")"), "false");
    assert_eq!(eval!("(lines \"a\nb\nc\")"), "(\"a\" \"b\" \"c\")");

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
    assert_eq!(eval!("(+ 9223372036854775807 0)"),    "9223372036854775807");   // -> int
    assert_eq!(eval!("(- 9223372036854775808 1)"),    "9223372036854775807");   // -> bigint
    assert_eq!(eval!("(+ 9223372036854775807 1)"),    "9223372036854775808");   // -> bigint
    assert_eq!(eval!("(+ 9223372036854775807 1.0)"),  "9223372036854776000.0"); // -> float
    assert_eq!(eval!("(+ 9223372036854775807 10)"),   "9223372036854775817");   // -> bigint
    assert_eq!(eval!("(* 9223372036854775807 10)"),  "92233720368547758070");   // -> bigint

    assert_eq!(eval!("(^ 2 16)"),                                      "65536");   // -> int
    assert_eq!(eval!("(^ 2 128)"),   "340282366920938463463374607431768211456");   // -> bigint
    assert_eq!(eval!("(^ 2.0 128)"), "340282366920938500000000000000000000000.0"); // -> float

    assert_eq!(eval!("(number-type 9223372036854775807)"),   "\"int\"");
    assert_eq!(eval!("(number-type 9223372036854775808)"),   "\"bigint\"");
    assert_eq!(eval!("(number-type 9223372036854776000.0)"), "\"float\"");

    // quasiquote
    eval!("(define x 'a)");
    assert_eq!(eval!("`(x ,x y)"), "(x a y)");
    assert_eq!(eval!("`(x ,x y ,(+ 1 2))"), "(x a y 3)");

    // unquote-splicing
    eval!("(define x '(1 2 3))");
    assert_eq!(eval!("`(+ ,x)"), "(+ (1 2 3))");
    assert_eq!(eval!("`(+ ,@x)"), "(+ 1 2 3)");

    // macro
    eval!("(define foo 42)");
    eval!("(define set-10 (macro (x) `(set ,x 10)))");
    eval!("(set-10 foo)");
    assert_eq!(eval!("foo"), "10");

    // dotted pair
    assert_eq!(eval!("(cons 1 (cons 2 (cons 3 '())))"), "(1 2 3)");
    assert_eq!(eval!("(cons 1 (2 . (3 . '())))"),       "(1 2 3)");
    assert_eq!(eval!("(cons 1 (list 2 3))"),            "(1 2 3)");
    assert_eq!(eval!("'(cons 1 (cons 2 (cons 3 '())))"), "(cons 1 (cons 2 (cons 3 (quote ()))))");
    assert_eq!(eval!("'(1 . (2 . (3 . '())))"),          "(cons 1 (cons 2 (cons 3 (quote ()))))");

    // args
    eval!("(define list* (function args (append args '())))");
    assert_eq!(eval!("(list* 1 2 3)"), "(1 2 3)");
    eval!("(define (list* . args) (append args '())))");
    assert_eq!(eval!("(list* 1 2 3)"), "(1 2 3)");
}
