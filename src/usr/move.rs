use crate::usr;

pub fn main(args: &[&str]) -> Result<usize, usize> {
    if args.len() != 3 {
        return Err(1);
    }

    // TODO: Avoid doing copy+delete
    match usr::copy::main(args) {
        Ok(0) => usr::delete::main(&args[0..2]),
        _ => Err(1),
    }
}
