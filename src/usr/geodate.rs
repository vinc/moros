use crate::api::process::ExitCode;
use crate::api::clock;
use alloc::format;

use geodate::geodate;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() < 2 {
        eprintln!("Usage: geodate <longitude> [<timestamp>]");
        return Err(ExitCode::UsageError);
    }

    let format = "%h:%y:%m:%d:%c:%b";
    let longitude = args[1].parse().expect("Could not parse longitude");
    let timestamp = if args.len() == 3 {
        args[2].parse().expect("Could not parse timestamp")
    } else {
        clock::realtime()
    };

    let t = geodate::get_formatted_date(format, timestamp as i64, longitude);
    println!("{}", t);

    Ok(())
}
