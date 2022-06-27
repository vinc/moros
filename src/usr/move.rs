use crate::api::process;
use crate::usr;

pub fn main(args: &[&str]) -> Result<(), usize> {
    if args.len() != 3 {
        return Err(process::EXIT_FAILURE);
    }

    // TODO: Avoid doing copy+delete
    match usr::copy::main(args) {
        Ok(()) => usr::delete::main(&args[0..2]),
        _ => Err(process::EXIT_FAILURE),
    }
}
