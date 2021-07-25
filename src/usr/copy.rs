use crate::{sys, usr};
use alloc::vec;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 3 {
        println!("Usage: copy <source> <dest>");
        return usr::shell::ExitCode::CommandError;
    }

    let source = args[1];
    let dest = args[2];

    if dest.starts_with("/dev") || dest.starts_with("/sys") {
        println!("Permission denied to write to '{}'", dest);
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
                    println!("Could not write to '{}'", dest);
                    usr::shell::ExitCode::CommandError
                }
            }
        } else {
            println!("Permission denied to write to '{}'", dest);
            usr::shell::ExitCode::CommandError
        }
    } else {
        println!("File not found '{}'", source);
        usr::shell::ExitCode::CommandError
    }
}
