use crate::user;
use crate::api::syscall;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() == 2 {
        if let Ok(duration) = args[1].parse::<f64>() {
            syscall::sleep(duration);
            return user::shell::ExitCode::CommandSuccessful;
        }
    }
    user::shell::ExitCode::CommandError
}
