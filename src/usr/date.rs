use crate::{api, usr};
use time::validate_format_string;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let format = if args.len() > 1 { args[1] } else { "%F %H:%M:%S" };
    match validate_format_string(format) {
        Ok(()) => {
            println!("{}", api::time::now().format(format));
            usr::shell::ExitCode::CommandSuccessful
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            usr::shell::ExitCode::CommandError
        }
    }
}
