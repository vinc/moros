use crate::usr;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 3 {
        return usr::shell::ExitCode::CommandError;
    }

    // TODO: Avoid doing copy+delete
    match usr::copy::main(args) {
        usr::shell::ExitCode::CommandSuccessful => usr::delete::main(&args[0..2]),
        _ => usr::shell::ExitCode::CommandError,
    }
}
