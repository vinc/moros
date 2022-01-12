use crate::usr;
use crate::api::syscall;
use alloc::format;

use geodate::geodate;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() < 2 {
        eprintln!("Usage: geodate <longitude> [<timestamp>]");
        return usr::shell::ExitCode::CommandError;
    }

    let format = "%h:%y:%m:%d:%c:%b";
    let longitude = args[1].parse().expect("Could not parse longitude");
    let timestamp = if args.len() == 3 {
        args[2].parse().expect("Could not parse timestamp")
    } else {
        syscall::realtime()
    };

    let t = geodate::get_formatted_date(format, longitude, timestamp);
    println!("{}", t);

    usr::shell::ExitCode::CommandSuccessful
}
