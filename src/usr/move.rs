use crate::api::process::ExitCode;
use crate::usr;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() != 3 {
        return Err(ExitCode::UsageError);
    }

    // TODO: Avoid doing copy+delete
    match usr::copy::main(args) {
        Ok(()) => usr::delete::main(&args[0..2]),
        _ => Err(ExitCode::Failure),
    }
}
