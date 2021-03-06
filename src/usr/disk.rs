use crate::{sys, usr};
use alloc::vec::Vec;

const COMMANDS: [&'static str; 2] = ["list", "format"];

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
    print!("Usage: <command>\n");
    print!("\n");
    print!("Commands:\n");
    print!("  list\n");
    print!("  format <path>\n");

    usr::shell::ExitCode::CommandError
}

fn list() -> usr::shell::ExitCode {
    print!("Path            Name (Size)\n");
    for (bus, drive, model, serial, size, unit) in sys::ata::list() {
        print!("/dev/ata/{}/{}    {} {} ({} {})\n", bus, drive, model, serial, size, unit);
    }
    usr::shell::ExitCode::CommandSuccessful
}

fn format(pathname: &str) -> usr::shell::ExitCode {
    let path: Vec<_> = pathname.split('/').collect();
    if !pathname.starts_with("/dev/ata/") || path.len() != 5 {
        print!("Could not find disk at '{}'\n", pathname);
        return usr::shell::ExitCode::CommandError;
    }

    let bus = path[3].parse().expect("Could not parse <bus>");
    let dsk = path[4].parse().expect("Could not parse <dsk>");
    sys::fs::format(bus, dsk);
    print!("Disk successfully formatted\n");
    print!("MFS is now mounted to '/'\n");

    usr::shell::ExitCode::CommandSuccessful
}
