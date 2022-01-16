use crate::{api, usr};

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 2 {
        return usr::shell::ExitCode::CommandError;
    }

    let pathname = args[1];

    if pathname.starts_with("/dev") || pathname.starts_with("/sys") {
        eprintln!("Permission denied to write to '{}'", pathname);
        return usr::shell::ExitCode::CommandError;
    }

    // The command `write /usr/alice/` with a trailing slash will create
    // a directory, while the same command without a trailing slash will
    // create a file.
    let res = if pathname.ends_with('/') {
        let pathname = pathname.trim_end_matches('/');
        api::fs::create_dir(pathname)
    } else {
        api::fs::create_file(pathname)
    };
    if let Some(handle) = res {
        api::syscall::close(handle);
        usr::shell::ExitCode::CommandSuccessful
    } else {
        eprintln!("Could not write to '{}'", pathname);
        usr::shell::ExitCode::CommandError
    }
}
