use crate::{sys, usr, print};

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        for (key, val) in sys::process::envs() {
            print!("{}={}\n", key, val);
        }
    } else {
        for arg in args[1..].iter() {
            if let Some(i) = arg.find('=') {
                let (key, mut val) = arg.split_at(i);
                val = &val[1..];
                sys::process::set_env(key, val);
                print!("{}={}\n", key, val);
            } else {
                print!("Error: could not parse '{}'\n", arg);
                return usr::shell::ExitCode::CommandError;
            }
        }
    }
    usr::shell::ExitCode::CommandSuccessful
}
