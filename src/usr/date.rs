use crate::api;
use crate::api::clock::DATE_TIME_ZONE;
use crate::api::process::ExitCode;
use time::validate_format_string;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    let format = if args.len() > 1 { args[1] } else { DATE_TIME_ZONE };
    match validate_format_string(format) {
        Ok(()) => {
            println!("{}", api::time::now().format(format));
            Ok(())
        }
        Err(e) => {
            error!("{}", e);
            Err(ExitCode::Failure)
        }
    }
}
