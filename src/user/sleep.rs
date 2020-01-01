use crate::{kernel, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() == 2 {
        if let Ok(duration) = args[1].parse::<f64>() {
            kernel::sleep::sleep(duration);
            return user::shell::ExitCode::CommandSuccessful;
        }
    }
    user::shell::ExitCode::CommandError
}
