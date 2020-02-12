use alloc::vec::Vec;
use crate::{print, kernel, user};

// Example: mkfs /dev/ata/bus/0/dsk/0
pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        print!("Usage: mkfs <device>\n");
        return user::shell::ExitCode::CommandError;
    }

    let path: Vec<_> = args[1].split('/').collect();

    if path.len() != 7 {
        print!("Could not recognize <device>\n");
        return user::shell::ExitCode::CommandError;
    }

    let bus = path[4].parse().expect("Could not parse <bus>");
    let dsk = path[6].parse().expect("Could not parse <dsk>");
    print!("Making filesystem on ATA {}:{} ...\n", bus, dsk);
    kernel::fs::make(bus, dsk);

    user::shell::ExitCode::CommandSuccessful
}
