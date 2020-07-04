use alloc::vec::Vec;
use crate::{print, kernel, user};

const COMMANDS: [&'static str; 1] = ["format"];

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() == 1 || !COMMANDS.contains(&args[1]) {
        return usage();
    }

    match args[1] {
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

fn usage() -> user::shell::ExitCode {
    print!("Usage: <command>\n");
    print!("\n");
    print!("Commands:\n");
    print!("  format /dev/ata/<bus>/<dsk>\n");

    user::shell::ExitCode::CommandError
}

pub fn format(pathname: &str) -> user::shell::ExitCode {
    let path: Vec<_> = pathname.split('/').collect();
    if !pathname.starts_with("/dev/ata/") || path.len() != 5 {
        print!("Could not recognize <device>\n");
        return user::shell::ExitCode::CommandError;
    }

    let bus = path[3].parse().expect("Could not parse <bus>");
    let dsk = path[4].parse().expect("Could not parse <dsk>");
    kernel::fs::format(bus, dsk);
    print!("Disk successfully formatted\n");
    print!("MFS mounted to '/'\n");

    user::shell::ExitCode::CommandSuccessful
}
