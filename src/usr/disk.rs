use crate::{sys, usr};
use alloc::vec::Vec;

const COMMANDS: [&str; 2] = ["list", "format"];

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 || !COMMANDS.contains(&args[1]) {
        return usage();
    }

    match args[1] {
        "list" => {
            list()
        },
        "format" => {
            if args.len() == 2 {
                return usage();
            }
            format(args[2])
        },
        _ => {
            usage()
        }
    }
}

fn usage() -> usr::shell::ExitCode {
    println!("Usage: <command>");
    println!();
    println!("Commands:");
    println!("  list");
    println!("  format <path>");

    usr::shell::ExitCode::CommandError
}

fn list() -> usr::shell::ExitCode {
    println!("Path            Name (Size)");
    for (bus, drive, model, serial, size, unit) in sys::ata::list() {
        println!("/dev/ata/{}/{}    {} {} ({} {})", bus, drive, model, serial, size, unit);
    }
    usr::shell::ExitCode::CommandSuccessful
}

fn format(pathname: &str) -> usr::shell::ExitCode {
    let path: Vec<_> = pathname.split('/').collect();
    if !pathname.starts_with("/dev/ata/") || path.len() != 5 {
        println!("Could not find disk at '{}'", pathname);
        return usr::shell::ExitCode::CommandError;
    }

    let bus = path[3].parse().expect("Could not parse <bus>");
    let dsk = path[4].parse().expect("Could not parse <dsk>");
    sys::fs::format(bus, dsk);
    println!("Disk successfully formatted");
    println!("MFS is now mounted to '/'");

    usr::shell::ExitCode::CommandSuccessful
}
