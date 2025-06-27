#![no_std]
#![no_main]

extern crate alloc;

use geodate::geodate;
use moros::entry_point;
use moros::{println, print};
use moros::api::clock;

entry_point!(main);

fn main(args: &[&str]) {
    if args.len() < 2 {
        println!("Usage: geodate <longitude> [<timestamp>]");
        return;
    }

    let fmt = "%h%y-%m-%d %c:%b";
    let longitude = args[1].parse().expect("Could not parse longitude");
    let timestamp = if args.len() == 3 {
        args[2].parse().expect("Could not parse timestamp")
    } else {
        clock::epoch_time()
    };
    let date = geodate::get_formatted_date(fmt, timestamp as i64, longitude);
    println!("{date}");
}
