use crate::{sys, usr, print};
use alloc::vec::Vec;

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

    if let Some(source_file) = sys::fs::File::open(source) {
        if let Some(mut dest_file) = sys::fs::File::create(dest) {
            let filesize = source_file.size();
            let mut buf = Vec::with_capacity(filesize);
            buf.resize(filesize, 0);
            source_file.read(&mut buf);
            match dest_file.write(&buf) {
                Ok(()) => {
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
