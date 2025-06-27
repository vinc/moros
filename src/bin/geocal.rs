#![no_std]
#![no_main]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use geodate::geodate::*;
use geodate::reverse::*;
use geodate::ephemeris::*;
use moros::entry_point;
use moros::{eprintln, eprint, println, print};
use moros::api::clock;

entry_point!(main);

fn main(args: &[&str]) {
    let mut show_ephemeris = false;
    let mut solar_calendar = false;
    let args: Vec<&&str> = args.iter().filter(|arg| {
        match **arg {
            "--ephem" => show_ephemeris = true,
            "--solar" => solar_calendar = true,
            _ => {},
        }
        !arg.starts_with("--")
    }).collect();

    if args.len() < 3 {
        eprintln!("Usage: geocal <latitude> <longitude> [<timestamp>]");
        return;
    }

    let latitude = args[1].parse().unwrap();
    let longitude = args[2].parse().unwrap();
    let timestamp = if args.len() == 4 {
        args[3].parse().unwrap()
    } else {
        clock::epoch_time() as i64
    };

    let week;
    let format;
    let last_day;
    if solar_calendar {
        week = 10;
        format = String::from("%h:%y:%s:%d:%c:%b");
        last_day = last_day_of_solar_month(timestamp, longitude);
    } else {
        week = 8;
        format = String::from("%h:%y:%m:%d:%c:%b");
        last_day = last_day_of_lunisolar_month(timestamp, longitude);
    };
    let formatted_date = get_formatted_date(&format, timestamp, longitude);
    let date: Vec<_> = formatted_date.split(":").collect();

    println!();
    let sep = "|";
    print_line(week);

    // Date
    let is_negative = date[0].starts_with('-');
    let colored_title = "Date";
    let colored_date = [
        "\x1b[91m", date[0], date[1], "-", date[2], "-", date[3], "\x1b[0m"
    ].join("");
    let mut spacing = (3 * week) - 17;
    if is_negative {
        spacing -= 1;
    }
    let space = " ".repeat(spacing);
    println!("  {sep} {colored_title} {space} {colored_date} {sep}");
    print_line(week);

    // Calendar
    let line = if solar_calendar {
        [" ", sep, "So Me Ve Te Ma Ju Sa Ur Ne Lu", ""].join(" ")
    } else {
        [" ", sep, "So Me Ve Te Ma Ju Sa Lu", ""].join(" ")
    };
    print!("{line}");
    let n = last_day + 1;
    for i in 0..n {
        // Weekend
        if solar_calendar {
            if i % week == 0 {
                print!("|\n  {sep} ");
            }
        } else if i == 0 || i == 7 || i == 15 || i == 22 {
            // The lunisolar calendar has a leap day at the end of the
            // second week and another at the end of the last week if
            // the month is long (30 days).
            if i == 7 || i == 22 {
                print!("   ");
            }
            print!("|\n  {sep} ");
        }

        let mut day = format!("{:02}", i);
        if day == date[3] {
            day = ["\x1b[91m", &day, "\x1b[0m"].join("");
        }
        print!("{day} ");
    }

    let n = if solar_calendar {
        (if last_day > 89 { 99 } else { 89 }) - last_day
    } else if last_day == 28 {
        1
    } else {
        0
    };
    let space = "   ".repeat(n);
    println!("{space}|");
    print_line(week);

    // Time
    let colored_title = "Time";
    let colored_time = ["\x1b[91m", date[4], ":", date[5], "\x1b[0m"].join("");
    let spacing = (3 * week) - 12;
    let space = " ".repeat(spacing);
    println!("  {sep} {colored_title} {space} {colored_time} {sep}");
    print_line(week);

    // Ephemeris
    if show_ephemeris {
        let events = get_ephemeris(timestamp, longitude, latitude);
        for (&t, e) in &events {
            let name = match e.as_str() {
                "Current" => continue,
                "First Quarter Moon" => "First Quarter",
                "Last Quarter Moon" => "Last Quarter",
                _ => e
            };
            let time = get_formatted_date("%c:%b", t, longitude);
            let spacing = (3 * week) - 8 - name.len();
            let space = " ".repeat(spacing);
            println!("  {sep} {name} {space} {time} {sep}");
        }
        print_line(week);
    }
}

// A lunisolar month can be 29 or 30 days long
fn last_day_of_lunisolar_month(timestamp: i64, longitude: f64) -> usize {
    // HACK: This rely on an undefined behavior when getting a timestamp for
    // day following the last day of the month.
    let format = String::from("%h:%y:%m:%d:%c:%b");
    let a = get_formatted_date("%h:%y:%m:29:50:00", timestamp, longitude);
    let t = get_timestamp(format.clone(), a.clone(), longitude);
    let b = get_formatted_date(&format, t, longitude);
    if a == b {
        29
    } else {
        28
    }
}

// A solar month can be 88 to 94 days long
fn last_day_of_solar_month(timestamp: i64, longitude: f64) -> usize {
    // HACK: This rely on an undefined behavior when getting a timestamp for
    // day following the last day of the month.
    let format = String::from("%h:%y:%s:%d:%c:%b");
    for i in 88..100 {
        let d = format!("{:02}", i);
        let f = ["%h:%y:%s:", &d, ":50:00"].join("");
        let a = get_formatted_date(&f, timestamp, longitude);
        let t = get_timestamp(format.clone(), a.clone(), longitude);
        let b = get_formatted_date(&format, t, longitude);
        if a != b {
            return i - 1;
        }
    }
    unreachable!();
}

fn print_line(week: usize) {
    let s = "-".repeat(3 * week);
    println!("  +-{s}+");
}
