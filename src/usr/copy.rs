use crate::{kernel, print, user};
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 3 {
        print!("Usage: copy <source> <dest>\n");
        return user::shell::ExitCode::CommandError;
    }

    let source = args[1];
    let dest = args[2];

    if dest.starts_with("/dev") || dest.starts_with("/sys") {
        print!("Permission denied to write to '{}'\n", dest);
        return user::shell::ExitCode::CommandError;
    }

    if let Some(source_file) = kernel::fs::File::open(source) {
        if let Some(mut dest_file) = kernel::fs::File::create(dest) {
            let filesize = source_file.size();
            let mut buf = Vec::with_capacity(filesize);
            buf.resize(filesize, 0);
            source_file.read(&mut buf);
            match dest_file.write(&buf) {
                Ok(()) => {
                    user::shell::ExitCode::CommandSuccessful
                },
                Err(()) => {
                    print!("Could not write to '{}'\n", dest);
                    user::shell::ExitCode::CommandError
                }
            }
        } else {
            print!("Permission denied to write to '{}'\n", dest);
            user::shell::ExitCode::CommandError
        }
    } else {
        print!("File not found '{}'\n", source);
        user::shell::ExitCode::CommandError
    }
}
