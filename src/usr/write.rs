use crate::{sys, usr};

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 2 {
        return usr::shell::ExitCode::CommandError;
    }

    let pathname = args[1];

    if pathname.starts_with("/dev") || pathname.starts_with("/sys") {
        println!("Permission denied to write to '{}'", pathname);
        return usr::shell::ExitCode::CommandError;
    }

    // The command `write /usr/alice/` with a trailing slash will create
    // a directory, while the same command without a trailing slash will
    // create a file.
    let success = if pathname.ends_with('/') {
        let pathname = pathname.trim_end_matches('/');
        sys::fs::Dir::create(pathname).is_some()
    } else {
        sys::fs::File::create(pathname).is_some()
    };

    if success {
        usr::shell::ExitCode::CommandSuccessful
    } else {
        println!("Could not write to '{}'", pathname);
        usr::shell::ExitCode::CommandError
    }
}
