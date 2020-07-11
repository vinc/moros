use crate::{kernel, print, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    let time = kernel::clock::uptime();
    if args.len() == 2 && args[1] == "--raw" {
        print!("{:.6}\n", time);
    } else if args.len() == 2 && args[1] == "--metric" {
        user::date::print_time_in_seconds(time);
    } else {
        user::date::print_time_in_days(time / 86400.0);
    }
    user::shell::ExitCode::CommandSuccessful
}
