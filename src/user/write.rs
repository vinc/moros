use crate::{kernel, print, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let pathname = args[1];

    if pathname.starts_with("/dev") || pathname.starts_with("/sys") {
        print!("Permission denied to write to '{}'\n", pathname);
        return user::shell::ExitCode::CommandError;
    }

    // The command `write /usr/alice/` with a trailing slash will create
    // a directory, while the same command without a trailing slash will
    // create a file.
    let success = if pathname.ends_with('/') {
        let pathname = pathname.trim_end_matches('/');
        kernel::fs::Dir::create(pathname).is_some()
    } else {
        kernel::fs::File::create(pathname).is_some()
    };

    if success {
        user::shell::ExitCode::CommandSuccessful
    } else {
        print!("Could not write to '{}'\n", pathname);
        user::shell::ExitCode::CommandError
    }
}
