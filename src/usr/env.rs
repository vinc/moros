use crate::{kernel, print, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() == 1 {
        for (key, val) in kernel::process::envs() {
            print!("{}={}\n", key, val);
        }
    } else {
        for arg in args[1..].iter() {
            if let Some(i) = arg.find('=') {
                let (key, mut val) = arg.split_at(i);
                val = &val[1..];
                kernel::process::set_env(key, val);
                print!("{}={}\n", key, val);
            } else {
                print!("Error: could not parse '{}'\n", arg);
                return user::shell::ExitCode::CommandError;
            }
        }
    }
    user::shell::ExitCode::CommandSuccessful
}
