use alloc::vec::Vec;
use crate::{print, kernel, user};

// Example: mkfs /dev/ata/0/0
pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        print!("Usage: mkfs /dev/ata/<bus>/<dsk>\n");
        return user::shell::ExitCode::CommandError;
    }

    let pathname = args[1];
    let path: Vec<_> = pathname.split('/').collect();
    if !pathname.starts_with("/dev/ata/") || path.len() != 5 {
        print!("Could not recognize <device>\n");
        return user::shell::ExitCode::CommandError;
    }

    let bus = path[3].parse().expect("Could not parse <bus>");
    let dsk = path[4].parse().expect("Could not parse <dsk>");
    kernel::fs::make(bus, dsk);
    print!("MFS mounted to '/'\n");

    user::shell::ExitCode::CommandSuccessful
}
