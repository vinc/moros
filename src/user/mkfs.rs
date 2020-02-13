use alloc::vec::Vec;
use crate::{print, kernel, user};

// Example: mkfs /dev/ata/0/0
pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        print!("Usage: mkfs /dev/ata/<bus>/<dsk>\n");
        return user::shell::ExitCode::CommandError;
    }

    let path: Vec<_> = args[1].split('/').collect();

    if path.len() != 5 {
        print!("Could not recognize <device>\n");
        return user::shell::ExitCode::CommandError;
    }

    let bus = path[3].parse().expect("Could not parse <bus>");
    let dsk = path[4].parse().expect("Could not parse <dsk>");
    kernel::fs::make(bus, dsk);
    print!("MFS setup on ATA {}:{}\n", bus, dsk);

    user::shell::ExitCode::CommandSuccessful
}
