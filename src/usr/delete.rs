use crate::{sys, usr};
use crate::api::fs;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 2 {
        return usr::shell::ExitCode::CommandError;
    }

    let mut pathname = args[1];

    if pathname.starts_with("/dev") || pathname.starts_with("/sys") {
        eprintln!("Permission denied to delete '{}'", pathname);
        return usr::shell::ExitCode::CommandError;
    }

    // The commands `delete /usr/alice/` and `delete /usr/alice` are equivalent,
    // but `delete /` should not be modified.
    if pathname.len() > 1 {
        pathname = pathname.trim_end_matches('/');
    }

    if let Some(dir) = sys::fs::Dir::open(pathname) {
        if dir.entries().count() == 0 {
            if sys::fs::Dir::delete(pathname).is_ok() {
                usr::shell::ExitCode::CommandSuccessful
            } else {
                eprintln!("Could not delete directory '{}'", pathname);
                usr::shell::ExitCode::CommandError
            }
        } else {
            eprintln!("Directory '{}' not empty", pathname);
            usr::shell::ExitCode::CommandError
        }
    } else if fs::exists(pathname) {
        if sys::fs::File::delete(pathname).is_ok() {
            usr::shell::ExitCode::CommandSuccessful
        } else {
            eprintln!("Could not delete file '{}'", pathname);
            usr::shell::ExitCode::CommandError
        }
    } else {
        eprintln!("File not found '{}'", pathname);
        usr::shell::ExitCode::CommandError
    }
}
