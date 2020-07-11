use crate::{kernel, print, user};
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 3 {
        return user::shell::ExitCode::CommandError;
    }

    let from = args[1];
    let to = args[2];

    if to.starts_with("/dev") || to.starts_with("/sys") {
        print!("Permission denied to write to '{}'\n", to);
        return user::shell::ExitCode::CommandError;
    }

    if let Some(file_from) = kernel::fs::File::open(from) {
        if let Some(mut file_to) = kernel::fs::File::create(to) {
            let filesize = file_from.size();
            let mut buf = Vec::with_capacity(filesize);
            buf.resize(filesize, 0);
            file_from.read(&mut buf);
            match file_to.write(&buf) {
                Ok(()) => {
                    user::shell::ExitCode::CommandSuccessful
                },
                Err(()) => {
                    print!("Could not write to '{}'\n", to);
                    user::shell::ExitCode::CommandError
                }
            }
        } else {
            print!("Permission denied to write to '{}'\n", to);
            user::shell::ExitCode::CommandError
        }
    } else {
        print!("File not found '{}'\n", from);
        user::shell::ExitCode::CommandError
    }
}
