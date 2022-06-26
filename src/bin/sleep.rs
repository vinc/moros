#![no_std]
#![no_main]

use moros::api::syscall;
use moros::entry_point;

entry_point!(main);

fn main(args: &[&str]) -> isize {
    if args.len() == 2 {
        if let Ok(duration) = args[1].parse::<f64>() {
            syscall::sleep(duration);
            return 0
        }
    }
    -1
}
