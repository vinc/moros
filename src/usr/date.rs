use crate::api;
use time::validate_format_string;

pub fn main(args: &[&str]) -> Result<usize, usize> {
    let format = if args.len() > 1 { args[1] } else { "%F %H:%M:%S" };
    match validate_format_string(format) {
        Ok(()) => {
            println!("{}", api::time::now().format(format));
            Ok(0)
        }
        Err(e) => {
            error!("{}", e);
            Err(1)
        }
    }
}
