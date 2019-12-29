use crate::print;
use crate::kernel::clock;
use crate::user::date::{print_time_in_seconds, print_time_in_days};

pub fn main(args: &[&str]) {
    let time = clock::clock_monotonic();
    if args.len() == 2 && args[1] == "--raw" {
        print!("{:.6}\n", time);
    } else if args.len() == 2 && args[1] == "--metric" {
        print_time_in_seconds(time);
    } else {
        print_time_in_days(time / 86400.0);
    }
}
