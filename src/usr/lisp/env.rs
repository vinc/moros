use super::primitive;
use super::FUNCTIONS;
use super::{Err, Exp, Number};

use crate::{could_not, expected};

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::f64::consts::PI;

const BUILTINS: [&str; 27] = [
    "quote",
    "quasiquote",
    "unquote",
    "unquote-splice",
    "atom?",
    "eq?",
    "cons",
    "if",
    "cond",
    "case",
    "fun",
    "var",
    "var?",
    "mut",
    "mac",
    "def",
    "def-fun",
    "def-mac",
    "eval",
    "expand",
    "apply",
    "fold",
    "while",
    "do",
    "load",
    "doc",
    "env",
];

#[derive(Clone)]
pub struct Env {
    pub data: BTreeMap<String, Exp>,
    pub outer: Option<Rc<RefCell<Env>>>,
}

pub fn default_env() -> Rc<RefCell<Env>> {
    let mut data: BTreeMap<String, Exp> = BTreeMap::new();

    data.insert(
        "pi".to_string(),
        Exp::Num(Number::from(PI)),
    );
    data.insert(
        "gt?".to_string(),
        Exp::Primitive(primitive::lisp_gt),
    );
    data.insert(
        "gte?".to_string(),
        Exp::Primitive(primitive::lisp_gte),
    );
    data.insert(
        "lt?".to_string(),
        Exp::Primitive(primitive::lisp_lt),
    );
    data.insert(
        "lte?".to_string(),
        Exp::Primitive(primitive::lisp_lte),
    );
    data.insert(
        "add".to_string(),
        Exp::Primitive(primitive::lisp_add),
    );
    data.insert(
        "sub".to_string(),
        Exp::Primitive(primitive::lisp_sub),
    );
    data.insert(
        "mul".to_string(),
        Exp::Primitive(primitive::lisp_mul),
    );
    data.insert(
        "div".to_string(),
        Exp::Primitive(primitive::lisp_div),
    );
    data.insert(
        "exp".to_string(),
        Exp::Primitive(primitive::lisp_exp),
    );
    data.insert(
        "rem".to_string(),
        Exp::Primitive(primitive::lisp_rem),
    );
    data.insert(
        "bit/and".to_string(),
        Exp::Primitive(primitive::lisp_bit_and),
    );
    data.insert(
        "bit/xor".to_string(),
        Exp::Primitive(primitive::lisp_bit_xor),
    );
    data.insert(
        "bit/or".to_string(),
        Exp::Primitive(primitive::lisp_bit_or),
    );
    data.insert(
        "bit/shl".to_string(),
        Exp::Primitive(primitive::lisp_bit_shl),
    );
    data.insert(
        "bit/shr".to_string(),
        Exp::Primitive(primitive::lisp_bit_shr),
    );
    data.insert(
        "cos".to_string(),
        Exp::Primitive(primitive::lisp_cos),
    );
    data.insert(
        "acos".to_string(),
        Exp::Primitive(primitive::lisp_acos),
    );
    data.insert(
        "asin".to_string(),
        Exp::Primitive(primitive::lisp_asin),
    );
    data.insert(
        "atan".to_string(),
        Exp::Primitive(primitive::lisp_atan),
    );
    data.insert(
        "sin".to_string(),
        Exp::Primitive(primitive::lisp_sin),
    );
    data.insert(
        "tan".to_string(),
        Exp::Primitive(primitive::lisp_tan),
    );
    data.insert(
        "sh".to_string(),
        Exp::Primitive(primitive::lisp_shell),
    );
    data.insert(
        "sh->bin".to_string(),
        Exp::Primitive(primitive::lisp_shell_binary),
    );
    data.insert(
        "str".to_string(),
        Exp::Primitive(primitive::lisp_string),
    );
    data.insert(
        "str->bin".to_string(),
        Exp::Primitive(primitive::lisp_string_binary),
    );
    data.insert(
        "bin->str".to_string(),
        Exp::Primitive(primitive::lisp_binary_string),
    );
    data.insert(
        "bin->num".to_string(),
        Exp::Primitive(primitive::lisp_binary_number),
    );
    data.insert(
        "num->bin".to_string(),
        Exp::Primitive(primitive::lisp_number_binary),
    );
    data.insert(
        "num->str".to_string(),
        Exp::Primitive(primitive::lisp_number_string),
    );
    data.insert(
        "str->num".to_string(),
        Exp::Primitive(primitive::lisp_string_number),
    );
    data.insert(
        "type".to_string(),
        Exp::Primitive(primitive::lisp_type),
    );
    data.insert(
        "parse".to_string(),
        Exp::Primitive(primitive::lisp_parse),
    );
    data.insert(
        "list".to_string(),
        Exp::Primitive(primitive::lisp_list),
    );
    data.insert(
        "sort".to_string(),
        Exp::Primitive(primitive::lisp_sort),
    );
    data.insert(
        "uniq".to_string(),
        Exp::Primitive(primitive::lisp_unique),
    );
    data.insert(
        "contains?".to_string(),
        Exp::Primitive(primitive::lisp_contains),
    );
    data.insert(
        "slice".to_string(),
        Exp::Primitive(primitive::lisp_slice),
    );
    data.insert(
        "chunks".to_string(),
        Exp::Primitive(primitive::lisp_chunks),
    );
    data.insert(
        "len".to_string(),
        Exp::Primitive(primitive::lisp_length),
    );
    data.insert(
        "concat".to_string(),
        Exp::Primitive(primitive::lisp_concat),
    );
    data.insert(
        "num/type".to_string(),
        Exp::Primitive(primitive::lisp_number_type),
    );
    data.insert(
        "num/int".to_string(),
        Exp::Primitive(primitive::lisp_number_int),
    );
    data.insert(
        "num/eq?".to_string(),
        Exp::Primitive(primitive::lisp_number_equal),
    );
    data.insert(
        "regex/find".to_string(),
        Exp::Primitive(primitive::lisp_regex_find),
    );
    data.insert(
        "str/split".to_string(),
        Exp::Primitive(primitive::lisp_string_split),
    );
    data.insert(
        "str/trim".to_string(),
        Exp::Primitive(primitive::lisp_string_trim),
    );
    data.insert(
        "file/size".to_string(),
        Exp::Primitive(primitive::lisp_file_size),
    );
    data.insert(
        "file/exists?".to_string(),
        Exp::Primitive(primitive::lisp_file_exists),
    );
    data.insert(
        "file/open".to_string(),
        Exp::Primitive(primitive::lisp_file_open),
    );
    data.insert(
        "file/read".to_string(),
        Exp::Primitive(primitive::lisp_file_read),
    );
    data.insert(
        "file/write".to_string(),
        Exp::Primitive(primitive::lisp_file_write),
    );
    data.insert(
        "file/close".to_string(),
        Exp::Primitive(primitive::lisp_file_close),
    );
    data.insert(
        "socket/connect".to_string(),
        Exp::Primitive(primitive::lisp_socket_connect),
    );
    data.insert(
        "socket/listen".to_string(),
        Exp::Primitive(primitive::lisp_socket_listen),
    );
    data.insert(
        "socket/accept".to_string(),
        Exp::Primitive(primitive::lisp_socket_accept),
    );
    data.insert(
        "host".to_string(),
        Exp::Primitive(primitive::lisp_host),
    );
    data.insert(
        "dict".to_string(),
        Exp::Primitive(primitive::lisp_dict),
    );
    data.insert(
        "dict/pairs".to_string(),
        Exp::Primitive(primitive::lisp_dict_pairs),
    );
    data.insert(
        "get".to_string(),
        Exp::Primitive(primitive::lisp_get),
    );
    data.insert(
        "put".to_string(),
        Exp::Primitive(primitive::lisp_put),
    );
    data.insert(
        "date".to_string(),
        Exp::Primitive(primitive::lisp_date),
    );
    data.insert(
        "sleep".to_string(),
        Exp::Primitive(primitive::lisp_sleep),
    );

    // Setup autocompletion
    *FUNCTIONS.lock() = data.keys().cloned().
        chain(BUILTINS.map(String::from)).collect();

    Rc::new(RefCell::new(Env { data, outer: None }))
}

pub fn env_keys(env: &Rc<RefCell<Env>>) -> Result<Vec<String>, Err> {
    let env = env.borrow_mut();
    let mut keys: Vec<String> = env.data.keys().cloned().collect();
    if let Some(outer_env) = &env.outer {
        keys.extend_from_slice(&env_keys(outer_env)?);
    }
    Ok(keys)
}

pub fn env_get(key: &str, env: &Rc<RefCell<Env>>) -> Result<Exp, Err> {
    let env = env.borrow_mut();
    match env.data.get(key) {
        Some(exp) => Ok(exp.clone()),
        None => match &env.outer {
            Some(outer_env) => env_get(key, outer_env),
            None => could_not!("find symbol {:?}", key),
        },
    }
}

pub fn env_set(
    key: &str,
    val: Exp,
    env: &Rc<RefCell<Env>>
) -> Result<Exp, Err> {
    let mut env = env.borrow_mut();
    match env.data.get(key) {
        Some(_) => {
            env.data.insert(key.to_string(), val.clone());
            Ok(val)
        }
        None => match &env.outer {
            Some(outer_env) => env_set(key, val, outer_env),
            None => could_not!("find symbol {:?}", key),
        },
    }
}

/// Bind arg values to param names in the new env returned.
pub fn bind(
    params: &Exp,
    args: &[Exp],
    outer: &mut Rc<RefCell<Env>>,
) -> Result<Rc<RefCell<Env>>, Err> {
    let mut args = args.to_vec();
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
                            if n - 1 <= m {
                                let rest = args.drain((n - 1)..).collect();
                                args.push(Exp::List(rest));
                            }
                        }
                    }
                }
            }
            let m = args.len();

            if m != n {
                let n = if is_variadic { n - 1 } else { n };        // Expect
                let s = if n != 1 { "s" } else { "" };              // Plural
                let a = if is_variadic { "at least " } else { "" }; // Prefix
                return expected!("{}{} argument{}, got {}", a, n, s, m);
            }
            for (exp, arg) in list.iter().zip(args.iter()) {
                if let Exp::Sym(s) = exp {
                    data.insert(s.clone(), arg.clone());
                } else {
                    return expected!("params to be a list of symbols");
                }
            }
        }
        _ => return expected!("params to be a list"),
    }
    Ok(Rc::new(RefCell::new(Env {
        data,
        outer: Some(outer.clone()),
    })))
}
