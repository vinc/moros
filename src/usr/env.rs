use crate::{sys, usr};

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        for (key, val) in sys::process::envs() {
            println!("{}={}", key, val);
        }
    } else {
        for arg in args[1..].iter() {
            if let Some(i) = arg.find('=') {
                let (key, mut val) = arg.split_at(i);
                val = &val[1..];
                sys::process::set_env(key, val);
                println!("{}={}", key, val);
            } else {
                eprintln!("Error: could not parse '{}'", arg);
                return usr::shell::ExitCode::CommandError;
            }
        }
    }
    usr::shell::ExitCode::CommandSuccessful
}
