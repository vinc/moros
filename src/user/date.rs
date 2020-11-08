use crate::{kernel, print, user};
use time::OffsetDateTime;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    let timestamp = kernel::clock::realtime();
    let format = if args.len() > 1 { args[1] } else { "%FT%T" };
    let date = OffsetDateTime::from_unix_timestamp(timestamp as i64);
    match time::util::validate_format_string(format) {
        Ok(()) => {
            print!("{}\n", date.format(format));
            user::shell::ExitCode::CommandSuccessful
        }
        Err(e) => {
            print!("Error: {}\n", e);
            user::shell::ExitCode::CommandError
        }
    }
}
