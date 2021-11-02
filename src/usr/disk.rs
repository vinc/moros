use crate::{sys, usr};
use crate::api::console::Style;
use alloc::string::ToString;
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() == 1 {
        return usage();
    }

    match args[1] {
        "format" => {
            if args.len() == 2 {
                return help();
            }
            format(args[2])
        },
        "usage" => {
            usage()
        },
        "list" => {
            list()
        },
        _ => {
            help()
        }
    }
}

fn help() -> usr::shell::ExitCode {
    println!("Usage: <command>");
    println!();
    println!("Commands:");
    println!("  format <path>");
    println!("  list");
    println!("  usage");

    usr::shell::ExitCode::CommandSuccessful
}

fn format(pathname: &str) -> usr::shell::ExitCode {
    let path: Vec<_> = pathname.split('/').collect();
    if !pathname.starts_with("/dev/ata/") || path.len() != 5 {
        eprintln!("Could not find disk at '{}'", pathname);
        return usr::shell::ExitCode::CommandError;
    }

    let bus = path[3].parse().expect("Could not parse <bus>");
    let dsk = path[4].parse().expect("Could not parse <dsk>");
    sys::fs::mount_ata(bus, dsk);
    sys::fs::format_ata();
    println!("Disk successfully formatted");
    println!("MFS is now mounted to '/'");

    usr::shell::ExitCode::CommandSuccessful
}

fn list() -> usr::shell::ExitCode {
    println!("Path            Name (Size)");
    for drive in sys::ata::list() {
        println!("/dev/ata/{}/{}    {}", drive.bus, drive.dsk, drive);
    }
    usr::shell::ExitCode::CommandSuccessful
}

fn usage() -> usr::shell::ExitCode {
    let size = sys::fs::disk_size();
    let used = sys::fs::disk_used();
    let free = size - used;

    let width = size.to_string().len();
    let color = Style::color("LightCyan");
    let reset = Style::reset();
    println!("{}Size:{} {:width$}", color, reset, size, width = width);
    println!("{}Used:{} {:width$}", color, reset, used, width = width);
    println!("{}Free:{} {:width$}", color, reset, free, width = width);
    usr::shell::ExitCode::CommandSuccessful
}
