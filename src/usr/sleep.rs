use crate::usr;
use crate::api::syscall;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 2 {
        if let Ok(duration) = args[1].parse::<f64>() {
            syscall::sleep(duration);
            return usr::shell::ExitCode::CommandSuccessful;
        }
    }
    usr::shell::ExitCode::CommandError
}
