#![no_std]
#![no_main]

use moros::api::syscall;
use moros::api::process;
use moros::entry_point;

entry_point!(main);

fn main(args: &[&str]) {
    if args.len() == 2 {
        if let Ok(duration) = args[1].parse::<f64>() {
            syscall::sleep(duration);
            return;
        } else {
            syscall::exit(process::EXIT_DATA_ERROR);
        }
    } else {
        syscall::exit(process::EXIT_USAGE_ERROR);
    }
}
