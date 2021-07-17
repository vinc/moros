use crate::{sys, usr, print};
use crate::api::console::Style;
use alloc::string::ToString;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::vec;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use core::fmt;
use core::num::ParseFloatError;

// Types

#[derive(Clone)]
enum RispExp {
    Bool(bool),
    Symbol(String),
    Number(f64),
    List(Vec<RispExp>),
    Func(fn(&[RispExp]) -> Result<RispExp, RispErr>),
    Lambda(RispLambda),
}

#[derive(Clone)]
struct RispLambda {
    params_exp: Rc<RispExp>,
    body_exp: Rc<RispExp>,
}

impl fmt::Display for RispExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = match self {
            RispExp::Bool(a) => a.to_string(),
            RispExp::Symbol(s) => s.clone(),
            RispExp::Number(n) => n.to_string(),
            RispExp::List(list) => {
                let xs: Vec<String> = list.iter().map(|x| x.to_string()).collect();
                format!("({})", xs.join(","))
            },
            RispExp::Func(_) => "Function {}".to_string(),
            RispExp::Lambda(_) => "Lambda {}".to_string(),
        };

        write!(f, "{}", str)
    }
}


#[derive(Debug)]
enum RispErr {
    Reason(String),
}

#[derive(Clone)]
struct RispEnv<'a> {
    data: BTreeMap<String, RispExp>,
    outer: Option<&'a RispEnv<'a>>,
}

// Parse

fn tokenize(expr: String) -> Vec<String> {
    expr.replace("(", " ( ").replace(")", " ) ").split_whitespace().map(|x| x.to_string()).collect()
}

fn parse<'a>(tokens: &'a [String]) -> Result<(RispExp, &'a [String]), RispErr> {
    let (token, rest) = tokens.split_first().ok_or(RispErr::Reason("could not get token".to_string()))?;
    match &token[..] {
        "(" => read_seq(rest),
        ")" => Err(RispErr::Reason("unexpected `)`".to_string())),
        _ => Ok((parse_atom(token), rest)),
    }
}

fn read_seq<'a>(tokens: &'a [String]) -> Result<(RispExp, &'a [String]), RispErr> {
    let mut res: Vec<RispExp> = vec![];
    let mut xs = tokens;
    loop {
        let (next_token, rest) = xs.split_first().ok_or(RispErr::Reason("could not find closing `)`".to_string()))?;
        if next_token == ")" {
            return Ok((RispExp::List(res), rest)) // skip `)`, head to the token after
        }
        let (exp, new_xs) = parse(&xs)?;
        res.push(exp);
        xs = new_xs;
    }
}

fn parse_atom(token: &str) -> RispExp {
    match token.as_ref() {
        "true" => RispExp::Bool(true),
        "false" => RispExp::Bool(false),
        _ => {
            let potential_float: Result<f64, ParseFloatError> = token.parse();
            match potential_float {
                Ok(v) => RispExp::Number(v),
                Err(_) => RispExp::Symbol(token.to_string().clone())
            }
        }
    }
}

// Env

macro_rules! ensure_tonicity {
    ($check_fn:expr) => {
        |args: &[RispExp]| -> Result<RispExp, RispErr> {
            let floats = parse_list_of_floats(args)?;
            let first = floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
            let rest = &floats[1..];
            fn f (prev: &f64, xs: &[f64]) -> bool {
                match xs.first() {
                    Some(x) => $check_fn(prev, x) && f(x, &xs[1..]),
                    None => true,
                }
            }
            Ok(RispExp::Bool(f(first, rest)))
        }
    };
}

fn default_env<'a>() -> RispEnv<'a> {
    let mut data: BTreeMap<String, RispExp> = BTreeMap::new();
    data.insert(
        "+".to_string(), 
        RispExp::Func(
            |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let sum = parse_list_of_floats(args)?.iter().fold(0.0, |sum, a| sum + a);
                Ok(RispExp::Number(sum))
            }
        )
    );
    data.insert(
        "-".to_string(), 
        RispExp::Func(
            |args: &[RispExp]| -> Result<RispExp, RispErr> {
                let floats = parse_list_of_floats(args)?;
                let first = *floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
                let sum_of_rest = floats[1..].iter().fold(0.0, |sum, a| sum + a);
                Ok(RispExp::Number(first - sum_of_rest))
            }
        )
    );
    data.insert(
        "=".to_string(), 
        RispExp::Func(ensure_tonicity!(|a, b| a == b))
    );
    data.insert(
        ">".to_string(), 
        RispExp::Func(ensure_tonicity!(|a, b| a > b))
    );
    data.insert(
        ">=".to_string(), 
        RispExp::Func(ensure_tonicity!(|a, b| a >= b))
    );
    data.insert(
        "<".to_string(), 
        RispExp::Func(ensure_tonicity!(|a, b| a < b))
    );
    data.insert(
        "<=".to_string(), 
        RispExp::Func(ensure_tonicity!(|a, b| a <= b))
    );

    RispEnv {data, outer: None}
}

fn parse_list_of_floats(args: &[RispExp]) -> Result<Vec<f64>, RispErr> {
    args.iter().map(|x| parse_single_float(x)).collect()
}

fn parse_single_float(exp: &RispExp) -> Result<f64, RispErr> {
    match exp {
        RispExp::Number(num) => Ok(*num),
        _ => Err(RispErr::Reason("expected a number".to_string())),
    }
}

// Eval

fn eval_if_args(arg_forms: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let test_form = arg_forms.first().ok_or(RispErr::Reason("expected test form".to_string()))?;
    let test_eval = eval(test_form, env)?;
    match test_eval {
        RispExp::Bool(b) => {
            let form_idx = if b { 1 } else { 2 };
            let res_form = arg_forms.get(form_idx).ok_or(RispErr::Reason(format!("expected form idx={}", form_idx)))?;
            eval(res_form, env)
        },
        _ => Err(RispErr::Reason(format!("unexpected test form='{}'", test_form.to_string())))
    }
}

fn eval_def_args(arg_forms: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let first_form = arg_forms.first().ok_or(
        RispErr::Reason(
            "expected first form".to_string(),
        )
    )?;
    let first_str = match first_form {
        RispExp::Symbol(s) => Ok(s.clone()),
        _ => Err(RispErr::Reason("expected first form to be a symbol".to_string()))
    }?;
    let second_form = arg_forms.get(1).ok_or(RispErr::Reason("expected second form".to_string()))?;
    if arg_forms.len() > 2 {
        return Err(RispErr::Reason("def can only have two forms ".to_string()))
    } 
    let second_eval = eval(second_form, env)?;
    env.data.insert(first_str, second_eval);
    Ok(first_form.clone())
}


fn eval_lambda_args(arg_forms: &[RispExp]) -> Result<RispExp, RispErr> {
    let params_exp = arg_forms.first().ok_or(RispErr::Reason("expected args form".to_string()))?;
    let body_exp = arg_forms.get(1).ok_or(RispErr::Reason("expected second form".to_string()))?;
    if arg_forms.len() > 2 {
        return Err(RispErr::Reason("fn definition can only have two forms ".to_string()))
    }
    Ok(RispExp::Lambda(RispLambda {
        body_exp: Rc::new(body_exp.clone()),
        params_exp: Rc::new(params_exp.clone()),
    }))
}


fn eval_built_in_form(exp: &RispExp, arg_forms: &[RispExp], env: &mut RispEnv) -> Option<Result<RispExp, RispErr>> {
    match exp {
        RispExp::Symbol(s) => {
            match s.as_ref() {
                "if" => Some(eval_if_args(arg_forms, env)),
                "def" => Some(eval_def_args(arg_forms, env)),
                "fn" => Some(eval_lambda_args(arg_forms)),
                _ => None,
            }
        },
        _ => None,
    }
}

fn env_get(k: &str, env: &RispEnv) -> Option<RispExp> {
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

fn parse_list_of_symbol_strings(form: Rc<RispExp>) -> Result<Vec<String>, RispErr> {
    let list = match form.as_ref() {
        RispExp::List(s) => Ok(s.clone()),
        _ => Err(RispErr::Reason("expected args form to be a list".to_string()))
    }?;
    list.iter().map(|x| {
        match x {
            RispExp::Symbol(s) => Ok(s.clone()),
            _ => Err(RispErr::Reason("expected symbols in the argument list".to_string()))
        }   
    }).collect()
}

fn env_for_lambda<'a>(params: Rc<RispExp>, arg_forms: &[RispExp], outer_env: &'a mut RispEnv) -> Result<RispEnv<'a>, RispErr> {
    let ks = parse_list_of_symbol_strings(params)?;
    if ks.len() != arg_forms.len() {
        return Err(RispErr::Reason(format!("expected {} arguments, got {}", ks.len(), arg_forms.len())));
    }
    let vs = eval_forms(arg_forms, outer_env)?;
    let mut data: BTreeMap<String, RispExp> = BTreeMap::new();
    for (k, v) in ks.iter().zip(vs.iter()) {
        data.insert(k.clone(), v.clone());
    }
    Ok(RispEnv {
        data,
        outer: Some(outer_env),
    })
}

fn eval_forms(arg_forms: &[RispExp], env: &mut RispEnv) -> Result<Vec<RispExp>, RispErr> {
    arg_forms.iter().map(|x| eval(x, env)).collect()
}

fn eval(exp: &RispExp, env: &mut RispEnv) -> Result<RispExp, RispErr> {
    match exp {
        RispExp::Symbol(k) => env_get(k, env).ok_or(RispErr::Reason(format!("unexpected symbol k='{}'", k))),
        RispExp::Bool(_a) => Ok(exp.clone()),
        RispExp::Number(_a) => Ok(exp.clone()),
        RispExp::List(list) => {
            let first_form = list.first().ok_or(RispErr::Reason("expected a non-empty list".to_string()))?;
            let arg_forms = &list[1..];
            match eval_built_in_form(first_form, arg_forms, env) {
                Some(res) => res,
                None => {
                    let first_eval = eval(first_form, env)?;
                    match first_eval {
                        RispExp::Func(f) => {
                            f(&eval_forms(arg_forms, env)?)
                        },
                        RispExp::Lambda(lambda) => {
                            let new_env = &mut env_for_lambda(lambda.params_exp, arg_forms, env)?;
                            eval(&lambda.body_exp, new_env)
                        },
                        _ => Err(RispErr::Reason("first form must be a function".to_string())),
                    }
                }
            }
        },
        RispExp::Func(_) => Err(RispErr::Reason("unexpected form".to_string())),
        RispExp::Lambda(_) => Err(RispErr::Reason("unexpected form".to_string())),
    }
}

// REPL

fn parse_eval(expr: String, env: &mut RispEnv) -> Result<RispExp, RispErr> {
    let (parsed_exp, _) = parse(&tokenize(expr))?;
    let evaled_exp = eval(&parsed_exp, env)?;
    Ok(evaled_exp)
}

fn slurp_expr() -> String {
    sys::console::get_line().trim_end().into()
}

pub fn main(_args: &[&str]) -> usr::shell::ExitCode {
    print!("MOROS Lisp v0.1.0\n\n");
    let csi_color = Style::color("Cyan");
    let csi_error = Style::color("Red");
    let csi_reset = Style::reset();
    let env = &mut default_env();
    loop {
        print!("{}>{} ", csi_color, csi_reset);
        let mut expr = slurp_expr();
        if !expr.starts_with("(") {
            expr = format!("({})", expr);
        }
        if expr == "(exit)" || sys::console::abort() {
            return usr::shell::ExitCode::CommandSuccessful;
        }
        match parse_eval(expr, env) {
            Ok(res) => print!("{}\n\n", res),
            Err(e) => match e {
                RispErr::Reason(msg) => print!("{}{}{}\n\n", csi_error, msg, csi_reset),
            },
        }
    }
}
