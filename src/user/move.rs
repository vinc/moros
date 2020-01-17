use crate::user;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 3 {
        return user::shell::ExitCode::CommandError;
    }

    // TODO: Avoid doing copy+delete
    match user::copy::main(args) {
        user::shell::ExitCode::CommandSuccessful => user::delete::main(&args[0..2]),
        _ => user::shell::ExitCode::CommandError,
    }
}
