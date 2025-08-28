#![no_std]
#![no_main]

extern crate alloc;

use geodate::geodate;
use moros::entry_point;
use moros::{println, print};
use moros::api::clock;
use moros::api::fs;
use moros::api::ini;
use moros::api::console::Style;

entry_point!(main);

const GEO_FILE: &str = "/ini/geo.ini";

fn main(args: &[&str]) {
    let mut longitude = None;
    let mut timestamp = None;
    let mut format = "%h%y-%m-%d %c:%b";

    let mut i = 1;
    let n = args.len();
    while i < n {
        match args[i] {
            "-h" | "--help" => {
                help();
                return;
            }
            "-f" | "--format" => {
                i += 1;
                if i < n {
                    format = args[i];
                }
            }
            "-x" | "--longitude" => {
                i += 1;
                if i < n {
                    if let Ok(value) = args[i].parse() {
                        longitude = Some(value);
                    }
                }
            }
            "-t" | "--timestamp" => {
                i += 1;
                if i < n {
                    if let Ok(value) = args[i].parse() {
                        timestamp = Some(value);
                    }
                }
            }
            _ => {
                help();
                return;
            }
        }
        i += 1;
    }

    if timestamp.is_none() {
        timestamp = Some(clock::epoch_time())
    }

    if longitude.is_none() {
        if let Ok(buf) = fs::read_to_string(GEO_FILE) {
            if let Some(config) = ini::parse(&buf) {
                if let Some(value) = config.get("lon") {
                    if let Ok(value) = value.parse() {
                        longitude = Some(value);
                    }
                }
            }
        }
    }

    if timestamp.is_none() || longitude.is_none() {
        help();
        return;
    }

    let f = format;
    let x = longitude.unwrap();
    let t = timestamp.unwrap() as i64;
    println!("{}", geodate::get_formatted_date(f, t, x));
}

fn help() {
    let csi_opt = Style::color("aqua");
    let csi_title = Style::color("yellow");
    let csi_reset = Style::reset();
    println!(
        "{}Usage:{} geodate {}<options>{1}", csi_title, csi_reset, csi_opt
    );
    println!();
    println!("{}Options:{}", csi_title, csi_reset);
    println!(
        "  {0}-f{1}, {0}--format <string>{1}       Set format",
        csi_opt, csi_reset
    );
    println!(
        "  {0}-t{1}, {0}--timestamp <number>{1}    Set timestamp",
        csi_opt, csi_reset
    );
    println!(
        "  {0}-x{1}, {0}--longitude <number>{1}    Set longitude",
        csi_opt, csi_reset
    );
}
