use crate::usr;
use crate::api::fs;

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

    if let Ok(contents) = fs::read(source) {
        if fs::write(dest, &contents).is_ok() {
            usr::shell::ExitCode::CommandSuccessful
        } else {
            println!("Could not write to '{}'", dest);
            usr::shell::ExitCode::CommandError
        }
    } else {
        println!("File not found '{}'", source);
        usr::shell::ExitCode::CommandError
    }
}
