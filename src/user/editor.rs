use crate::{print, kernel, user};
use heapless::String;
use heapless::consts::*;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let pathname = args[1];
    if pathname.starts_with("/dev") || pathname.starts_with("/sys") {
        print!("Permission denied to write to '{}'\n", pathname);
        user::shell::ExitCode::CommandError
    } else if let Some(mut file) = kernel::fs::File::create(pathname) {
        let contents = input();
        file.write(&contents);
        user::shell::ExitCode::CommandSuccessful
    } else {
        print!("Could not write to '{}'\n", pathname);
        user::shell::ExitCode::CommandError
    }
}

fn input() -> String<U2048> {
    let mut output = String::new();
    loop {
        let line = kernel::console::get_line();
        if line == ".\n" {
            break;
        }
        output.push_str(&line).ok(); // TODO: File full
    }
    output
}
