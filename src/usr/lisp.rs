use crate::{sys, usr};
use crate::api::console::Style;
use crate::api::prompt::Prompt;
use alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::vec;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use core::fmt;
use core::num::ParseFloatError;

// Adapted from Risp
// Copyright 2019 Stepan Parunashvili
// https://github.com/stopachka/risp
//
// See "Recursive Functions of Symbolic Expressions and Their Computation by Machine" by John McCarthy (1960)
// And "The Roots of Lisp" by Paul Graham (2002)
//
// MOROS Lisp is also inspired by Racket and Clojure

// Types

#[derive(Clone)]
enum Exp {
    Bool(bool),
    Symbol(String),
    Number(f64),
    List(Vec<Exp>),
    Func(fn(&[Exp]) -> Result<Exp, Err>),
    Lambda(Lambda),
}

#[derive(Clone)]
struct Lambda {
    params_exp: Rc<Exp>,
    body_exp: Rc<Exp>,
}

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            Exp::Bool(a) => a.to_string(),
            Exp::Symbol(s) => s.clone(),
            Exp::Number(n) => n.to_string(),
            Exp::List(list) => {
                let xs: Vec<String> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(" "))
            },
            Exp::Func(_) => "Function {}".to_string(),
            Exp::Lambda(_) => "Lambda {}".to_string(),
        };

        write!(f, "{}", str)
    }
}

#[derive(Debug)]
enum Err {
    Reason(String),
}

#[derive(Clone)]
struct Env<'a> {
    data: BTreeMap<String, Exp>,
    outer: Option<&'a Env<'a>>,
}

// Parse

fn tokenize(expr: &str) -> Vec<String> {
    expr.replace("(", " ( ")
        .replace(")", " ) ")
        .replace("'", " ' ")
        .split_whitespace().map(|x| x.to_string()).collect()
}

fn parse<'a>(tokens: &'a [String]) -> Result<(Exp, &'a [String]), Err> {
    let (token, rest) = tokens.split_first().ok_or(Err::Reason("could not get token".to_string()))?;
    match &token[..] {
        "'" => parse_quoted(rest),
        "(" => read_seq(rest),
        ")" => Err(Err::Reason("unexpected `)`".to_string())),
        _ => Ok((parse_atom(token), rest)),
    }
}

fn read_seq<'a>(tokens: &'a [String]) -> Result<(Exp, &'a [String]), Err> {
    let mut res: Vec<Exp> = vec![];
    let mut xs = tokens;
    loop {
        let (next_token, rest) = xs.split_first().ok_or(Err::Reason("could not find closing `)`".to_string()))?;
        if next_token == ")" {
            return Ok((Exp::List(res), rest)) // skip `)`, head to the token after
        }
        let (exp, new_xs) = parse(&xs)?;
        res.push(exp);
        xs = new_xs;
    }
}

fn parse_quoted<'a>(tokens: &'a [String]) -> Result<(Exp, &'a [String]), Err> {
    let xs = tokens;
    let (next_token, _) = xs.split_first().ok_or(Err::Reason("could not parse quote".to_string()))?;
    let (exp, rest) = if next_token == "(" {
        read_seq(&tokens[1..])? // Skip "("
    } else {
        parse(&tokens)?
    };
    let list = vec![Exp::Symbol("quote".to_string()), exp];
    Ok((Exp::List(list), rest))
}

fn parse_atom(token: &str) -> Exp {
    match token.as_ref() {
        "true" => Exp::Bool(true),
        "false" => Exp::Bool(false),
        _ => {
            let potential_float: Result<f64, ParseFloatError> = token.parse();
            match potential_float {
                Ok(v) => Exp::Number(v),
                Err(_) => Exp::Symbol(token.to_string().clone())
            }
        }
    }
}

// Env

macro_rules! ensure_tonicity {
    ($check_fn:expr) => {
        |args: &[Exp]| -> Result<Exp, Err> {
            let floats = parse_list_of_floats(args)?;
            let first = floats.first().ok_or(Err::Reason("expected at least one number".to_string()))?;
            let rest = &floats[1..];
            fn f (prev: &f64, xs: &[f64]) -> bool {
                match xs.first() {
                    Some(x) => $check_fn(prev, x) && f(x, &xs[1..]),
                    None => true,
                }
            }
            Ok(Exp::Bool(f(first, rest)))
        }
    };
}

fn default_env<'a>() -> Env<'a> {
    let mut data: BTreeMap<String, Exp> = BTreeMap::new();
    data.insert(
        "*".to_string(),
        Exp::Func(
            |args: &[Exp]| -> Result<Exp, Err> {
                let res = parse_list_of_floats(args)?.iter().fold(1.0, |res, a| res * a);
                Ok(Exp::Number(res))
            }
        )
    );
    data.insert(
        "+".to_string(), 
        Exp::Func(
            |args: &[Exp]| -> Result<Exp, Err> {
                let res = parse_list_of_floats(args)?.iter().fold(0.0, |res, a| res + a);
                Ok(Exp::Number(res))
            }
        )
    );
    data.insert(
        "-".to_string(), 
        Exp::Func(
            |args: &[Exp]| -> Result<Exp, Err> {
                let floats = parse_list_of_floats(args)?;
                let first = *floats.first().ok_or(Err::Reason("expected at least one number".to_string()))?;
                let sum_of_rest = floats[1..].iter().fold(0.0, |sum, a| sum + a);
                Ok(Exp::Number(first - sum_of_rest))
            }
        )
    );
    data.insert(
        "=".to_string(), 
        Exp::Func(ensure_tonicity!(|a, b| a == b))
    );
    data.insert(
        ">".to_string(), 
        Exp::Func(ensure_tonicity!(|a, b| a > b))
    );
    data.insert(
        ">=".to_string(), 
        Exp::Func(ensure_tonicity!(|a, b| a >= b))
    );
    data.insert(
        "<".to_string(), 
        Exp::Func(ensure_tonicity!(|a, b| a < b))
    );
    data.insert(
        "<=".to_string(), 
        Exp::Func(ensure_tonicity!(|a, b| a <= b))
    );

    Env {data, outer: None}
}

fn parse_list_of_floats(args: &[Exp]) -> Result<Vec<f64>, Err> {
    args.iter().map(|x| parse_single_float(x)).collect()
}

fn parse_single_float(exp: &Exp) -> Result<f64, Err> {
    match exp {
        Exp::Number(num) => Ok(*num),
        _ => Err(Err::Reason("expected a number".to_string())),
    }
}

// Eval

fn eval_quote_args(arg_forms: &[Exp]) -> Result<Exp, Err> {
    let first_form = arg_forms.first().ok_or(Err::Reason("expected first form".to_string()))?;
    Ok(first_form.clone())
}

fn eval_atom_args(arg_forms: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    let first_form = arg_forms.first().ok_or(Err::Reason("expected first form".to_string()))?;
    let first_eval = eval(first_form, env)?;
    match first_eval {
        Exp::Symbol(_) => Ok(Exp::Bool(true)),
        _              => Ok(Exp::Bool(false)),
    }
}

fn eval_eq_args(arg_forms: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    let first_form = arg_forms.first().ok_or(Err::Reason("expected first form".to_string()))?;
    let first_eval = eval(first_form, env)?;
    let second_form = arg_forms.get(1).ok_or(Err::Reason("expected second form".to_string()))?;
    let second_eval = eval(second_form, env)?;
    match first_eval {
        Exp::Symbol(a) => {
            match second_eval {
                Exp::Symbol(b) => {
                    Ok(Exp::Bool(a == b))
                },
                _ => Ok(Exp::Bool(false))
            }
        },
        Exp::List(a) => {
            match second_eval {
                Exp::List(b) => {
                    Ok(Exp::Bool(a.len() == 0 && b.len() == 0))
                },
                _ => Ok(Exp::Bool(false))
            }
        },
        _ => Ok(Exp::Bool(false))
    }
}

fn eval_car_args(arg_forms: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    let first_form = arg_forms.first().ok_or(Err::Reason("expected first form".to_string()))?;
    let first_eval = eval(first_form, env)?;
    match first_eval {
        Exp::List(list) => {
            let exp = list.first().ok_or(Err::Reason("list cannot be empty".to_string()))?; // TODO: return nil?
            Ok(exp.clone())
        },
        _ => Err(Err::Reason("expected list form".to_string())),
    }
}

fn eval_cdr_args(arg_forms: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    let first_form = arg_forms.first().ok_or(Err::Reason("expected first form".to_string()))?;
    let first_eval = eval(first_form, env)?;
    match first_eval {
        Exp::List(list) => {
            if list.len() < 1 {
                return Err(Err::Reason("list cannot be empty".to_string())) // TODO: return nil?
            }
            Ok(Exp::List(list[1..].iter().map(|exp| exp.clone()).collect()))
        },
        _ => Err(Err::Reason("expected list form".to_string())),
    }
}

fn eval_cons_args(arg_forms: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    let first_form = arg_forms.first().ok_or(Err::Reason("expected first form".to_string()))?;
    let first_eval = eval(first_form, env)?;
    let second_form = arg_forms.get(1).ok_or(Err::Reason("expected second form".to_string()))?;
    let second_eval = eval(second_form, env)?;
    match second_eval {
        Exp::List(mut list) => {
            list.insert(0, first_eval);
            Ok(Exp::List(list.iter().map(|exp| exp.clone()).collect()))
        },
        _ => Err(Err::Reason("expected list form".to_string())),
    }
}

fn eval_cond_args(arg_forms: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    if arg_forms.len() == 0 {
        return Err(Err::Reason("expected at least one form".to_string()))
    }
    for arg_form in arg_forms {
        match arg_form {
            Exp::List(list) => {
                if list.len() != 2 {
                    return Err(Err::Reason("expected lists of predicate and expression".to_string()))
                }
                let pred = eval(&list[0], env)?;
                let exp = eval(&list[1], env)?;
                match pred {
                    Exp::Bool(b) => {
                        if b {
                            return Ok(exp.clone());
                        }
                    },
                    _ => continue,
                }
            },
            _ => return Err(Err::Reason("expected lists of predicate and expression".to_string())),
        }
    }
    Ok(Exp::List(Vec::new()))
}

fn eval_label_args(arg_forms: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    let first_form = arg_forms.first().ok_or(Err::Reason("expected first form".to_string()))?;
    let first_str = match first_form {
        Exp::Symbol(s) => Ok(s.clone()),
        _ => Err(Err::Reason("expected first form to be a symbol".to_string()))
    }?;
    let second_form = arg_forms.get(1).ok_or(Err::Reason("expected second form".to_string()))?;
    if arg_forms.len() > 2 {
        return Err(Err::Reason("label can only have two forms".to_string()))
    } 
    let second_eval = eval(second_form, env)?;
    env.data.insert(first_str, second_eval);
    Ok(first_form.clone())
}

fn eval_lambda_args(arg_forms: &[Exp]) -> Result<Exp, Err> {
    let params_exp = arg_forms.first().ok_or(Err::Reason("expected args form".to_string()))?;
    let body_exp = arg_forms.get(1).ok_or(Err::Reason("expected second form".to_string()))?;
    if arg_forms.len() > 2 {
        return Err(Err::Reason("lambda definition can only have two forms".to_string()))
    }
    Ok(Exp::Lambda(Lambda {
        body_exp: Rc::new(body_exp.clone()),
        params_exp: Rc::new(params_exp.clone()),
    }))
}

fn eval_defun_args(arg_forms: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    let name = arg_forms.get(0).ok_or(Err::Reason("expected first form".to_string()))?.clone();
    let params = arg_forms.get(1).ok_or(Err::Reason("expected second form".to_string()))?.clone();
    let exp = arg_forms.get(2).ok_or(Err::Reason("expected third form".to_string()))?.clone();
    let lambda_args = vec![Exp::Symbol("lambda".to_string()), params, exp];
    let label_args = vec![name, Exp::List(lambda_args)];
    eval_label_args(&label_args, env)
}

fn eval_print_args(arg_forms: &[Exp], env: &mut Env) -> Result<Exp, Err> {
    let first_form = arg_forms.first().ok_or(Err::Reason("expected first form".to_string()))?;
    if arg_forms.len() > 1 {
        return Err(Err::Reason("print can only have one form".to_string()))
    }
    match eval(first_form, env) {
        Ok(res) => {
            println!("{}", res);
            Ok(res)
        },
        Err(res) => {
            Err(res)
        },
    }
}

fn eval_built_in_form(exp: &Exp, arg_forms: &[Exp], env: &mut Env) -> Option<Result<Exp, Err>> {
    match exp {
        Exp::Symbol(s) => {
            match s.as_ref() {
                // Seven Primitive Operators
                "quote"          => Some(eval_quote_args(arg_forms)),
                "atom" | "atom?" => Some(eval_atom_args(arg_forms, env)),
                "eq" | "eq?"     => Some(eval_eq_args(arg_forms, env)),
                "car" | "first"  => Some(eval_car_args(arg_forms, env)),
                "cdr" | "rest"   => Some(eval_cdr_args(arg_forms, env)),
                "cons"           => Some(eval_cons_args(arg_forms, env)),
                "cond"           => Some(eval_cond_args(arg_forms, env)),

                // Two Special Forms
                "label" | "def"  => Some(eval_label_args(arg_forms, env)),
                "lambda" | "fn"  => Some(eval_lambda_args(arg_forms)),

                "defun" | "defn" => Some(eval_defun_args(arg_forms, env)),
                "print"          => Some(eval_print_args(arg_forms, env)),
                _                => None,
            }
        },
        _ => None,
    }
}

fn env_get(k: &str, env: &Env) -> Option<Exp> {
    match env.data.get(k) {
        Some(exp) => Some(exp.clone()),
        None => {
            match &env.outer {
                Some(outer_env) => env_get(k, &outer_env),
                None => None
            }
        }
    }
}

fn parse_list_of_symbol_strings(form: Rc<Exp>) -> Result<Vec<String>, Err> {
    let list = match form.as_ref() {
        Exp::List(s) => Ok(s.clone()),
        _ => Err(Err::Reason("expected args form to be a list".to_string()))
    }?;
    list.iter().map(|x| {
        match x {
            Exp::Symbol(s) => Ok(s.clone()),
            _ => Err(Err::Reason("expected symbols in the argument list".to_string()))
        }   
    }).collect()
}

fn env_for_lambda<'a>(params: Rc<Exp>, arg_forms: &[Exp], outer_env: &'a mut Env) -> Result<Env<'a>, Err> {
    let ks = parse_list_of_symbol_strings(params)?;
    if ks.len() != arg_forms.len() {
        return Err(Err::Reason(format!("expected {} arguments, got {}", ks.len(), arg_forms.len())));
    }
    let vs = eval_forms(arg_forms, outer_env)?;
    let mut data: BTreeMap<String, Exp> = BTreeMap::new();
    for (k, v) in ks.iter().zip(vs.iter()) {
        data.insert(k.clone(), v.clone());
    }
    Ok(Env {
        data,
        outer: Some(outer_env),
    })
}

fn eval_forms(arg_forms: &[Exp], env: &mut Env) -> Result<Vec<Exp>, Err> {
    arg_forms.iter().map(|x| eval(x, env)).collect()
}

fn eval(exp: &Exp, env: &mut Env) -> Result<Exp, Err> {
    match exp {
        Exp::Symbol(k) => env_get(k, env).ok_or(Err::Reason(format!("unexpected symbol k='{}'", k))),
        Exp::Bool(_a) => Ok(exp.clone()),
        Exp::Number(_a) => Ok(exp.clone()),
        Exp::List(list) => {
            let first_form = list.first().ok_or(Err::Reason("expected a non-empty list".to_string()))?;
            let arg_forms = &list[1..];
            match eval_built_in_form(first_form, arg_forms, env) {
                Some(res) => res,
                None => {
                    let first_eval = eval(first_form, env)?;
                    match first_eval {
                        Exp::Func(f) => {
                            f(&eval_forms(arg_forms, env)?)
                        },
                        Exp::Lambda(lambda) => {
                            let new_env = &mut env_for_lambda(lambda.params_exp, arg_forms, env)?;
                            eval(&lambda.body_exp, new_env)
                        },
                        _ => Err(Err::Reason("first form must be a function".to_string())),
                    }
                }
            }
        },
        Exp::Func(_) => Err(Err::Reason("unexpected form".to_string())),
        Exp::Lambda(_) => Err(Err::Reason("unexpected form".to_string())),
    }
}

// REPL

fn parse_eval(expr: &str, env: &mut Env) -> Result<Exp, Err> {
    let (parsed_exp, _) = parse(&tokenize(expr))?;
    let evaled_exp = eval(&parsed_exp, env)?;
    Ok(evaled_exp)
}

fn strip_comments(s: &str) -> String {
    s.split("#").next().unwrap().into()
}


const COMPLETER_FORMS: [&str; 11] = [
    "quote", "atom?", "eq?", "first", "rest", "cons", "cond", "def", "fn",
    "defn", "print",
];

fn lisp_completer(line: &str) -> Vec<String> {
    let mut entries = Vec::new();
    if let Some(last_word) = line.split_whitespace().next_back() {
        if last_word.starts_with("(") {
            let f = &last_word[1..];
            for form in COMPLETER_FORMS {
                if form.starts_with(f) {
                    entries.push(form[f.len()..].to_string());
                }
            }
        }
    }
    entries
}

fn repl(env: &mut Env) -> usr::shell::ExitCode {
    print!("MOROS Lisp v0.1.0\n\n");
    let csi_color = Style::color("Cyan");
    let csi_error = Style::color("Red");
    let csi_reset = Style::reset();
    let mut prompt = Prompt::new();
    let history_file = "~/.lisp-history";
    prompt.history.load(history_file);
    prompt.completion.set(&lisp_completer);
    while let Some(exp) = prompt.input(&format!("{}>{} ", csi_color, csi_reset)) {
        if exp == "(exit)" {
            break;
        }
        match parse_eval(&exp, env) {
            Ok(res) => {
                print!("{}\n\n", res);
            }
            Err(e) => match e {
                Err::Reason(msg) => print!("{}Error: {}{}\n\n", csi_error, msg, csi_reset),
            },
        }
        if exp.len() > 0 {
            prompt.history.add(&exp);
            prompt.history.save(history_file);
        }
    }
    usr::shell::ExitCode::CommandSuccessful
}

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let env = &mut default_env();
    match args.len() {
        1 => {
            return repl(env);
        },
        2 => {
            let pathname = args[1];
            if let Some(mut file) = sys::fs::File::open(pathname) {
                let mut block = String::new();
                let mut opened = 0;
                let mut closed = 0;
                for line in file.read_to_string().split("\n") {
                    let line = strip_comments(line);
                    if line.len() > 0 {
                        opened += line.matches("(").count();
                        closed += line.matches(")").count();
                        block.push_str(&line);
                        if closed >= opened {
                            //println!("eval: '{}'", block);
                            if let Err(e) = parse_eval(&block, env) {
                                match e {
                                    Err::Reason(msg) => {
                                        println!("{}", msg);
                                        return usr::shell::ExitCode::CommandError;
                                    }
                                }
                            }
                            block.clear();
                            opened = 0;
                            closed = 0;
                        }
                    }
                }
                usr::shell::ExitCode::CommandSuccessful
            } else {
                print!("File not found '{}'\n", pathname);
                usr::shell::ExitCode::CommandError
            }
        },
        _ => {
            usr::shell::ExitCode::CommandError
        },
    }
}

#[test_case]
pub fn test_lisp() {
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
    assert_eq!(eval!("(atom 1)"), "false");

    // eq
    assert_eq!(eval!("(eq (quote a) (quote a))"), "true");
    assert_eq!(eval!("(eq (quote a) (quote b))"), "false");
    assert_eq!(eval!("(eq (quote a) (quote ()))"), "false");
    assert_eq!(eval!("(eq (quote ()) (quote ()))"), "true");

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
    assert_eq!(eval!("(cond ((< 2 4) 1) (true 2))"), "1");
    assert_eq!(eval!("(cond ((> 2 4) 1) (true 2))"), "2");

    // label
    eval!("(label a 2)");
    assert_eq!(eval!("(+ a 1)"), "3");
    //eval!("(label fn lambda)");
    //assert_eq!(eval!("((fn (a) (+ 1 a)) 2)"), "3");
    eval!("(label add-one (lambda (b) (+ b 1)))");
    assert_eq!(eval!("(add-one 2)"), "3");
    eval!("(label fib (lambda (n) (cond ((< n 2) n) (true (+ (fib (- n 1)) (fib (- n 2)))))))");
    assert_eq!(eval!("(fib 6)"), "8");

    // lambda
    assert_eq!(eval!("((lambda (a) (+ 1 a)) 2)"), "3");
    assert_eq!(eval!("((lambda (a) (* a a)) 2)"), "4");
    assert_eq!(eval!("((lambda (x) (cons x '(b c))) 'a)"), "(a b c)");

    // defun
    eval!("(defun add (a b) (+ a b))");
    assert_eq!(eval!("(add 1 2)"), "3");

    // addition
    assert_eq!(eval!("(+ 2 2)"), "4");
    assert_eq!(eval!("(+ 2 3 4)"), "9");
    assert_eq!(eval!("(+ 2 (+ 3 4))"), "9");

    // multiplication
    assert_eq!(eval!("(* 2 2)"), "4");
    assert_eq!(eval!("(* 2 3 4)"), "24");
    assert_eq!(eval!("(* 2 (* 3 4))"), "24");

    // comparisons
    assert_eq!(eval!("(< 6 4)"), "false");
    assert_eq!(eval!("(> 6 4 3 1)"), "true");
    assert_eq!(eval!("(= 6 4)"), "false");
    assert_eq!(eval!("(= 6 6)"), "true");
}
