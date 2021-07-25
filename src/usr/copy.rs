use crate::{sys, usr};
use alloc::vec;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 3 {
        print!("Usage: copy <source> <dest>\n");
        return usr::shell::ExitCode::CommandError;
    }

    let source = args[1];
    let dest = args[2];

    if dest.starts_with("/dev") || dest.starts_with("/sys") {
        print!("Permission denied to write to '{}'\n", dest);
        return usr::shell::ExitCode::CommandError;
    }

    if let Some(mut source_file) = sys::fs::File::open(source) {
        if let Some(mut dest_file) = sys::fs::File::create(dest) {
            let mut buf = vec![0; source_file.size()];
            source_file.read(&mut buf);
            match dest_file.write(&buf) {
                Ok(_) => {
                    usr::shell::ExitCode::CommandSuccessful
                },
                Err(()) => {
                    print!("Could not write to '{}'\n", dest);
                    usr::shell::ExitCode::CommandError
                }
            }
        } else {
            print!("Permission denied to write to '{}'\n", dest);
            usr::shell::ExitCode::CommandError
        }
    } else {
        print!("File not found '{}'\n", source);
        usr::shell::ExitCode::CommandError
    }
}
