use crate::{sys, usr};

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    match args.len() {
        1 => {
            for (key, val) in sys::process::envs() {
                println!("{:10} \"{}\"", key, val);
            }
            usr::shell::ExitCode::CommandSuccessful
        }
        2 => {
            let key = args[1];
            if let Some(val) = sys::process::env(key) {
                println!("{}", val);
                usr::shell::ExitCode::CommandSuccessful
            } else {
                error!("Could not get '{}'", key);
                usr::shell::ExitCode::CommandError
            }
        }
        3 => {
            sys::process::set_env(args[1], args[2]);
            usr::shell::ExitCode::CommandSuccessful
        }
        _ => {
            error!("Invalid number of arguments");
            usr::shell::ExitCode::CommandError
        }
    }
}
