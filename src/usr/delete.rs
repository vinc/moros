use crate::usr;
use crate::api::syscall;
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

    if !fs::exists(pathname) {
        eprintln!("File not found '{}'", pathname);
        return usr::shell::ExitCode::CommandError;
    }

    if let Some(stat) = syscall::stat(pathname) {
        if stat.is_dir() && stat.size() > 0 {
            eprintln!("Directory '{}' not empty", pathname);
            return usr::shell::ExitCode::CommandError;
        }
    }

    if fs::delete(pathname).is_ok() {
        usr::shell::ExitCode::CommandSuccessful
    } else {
        eprintln!("Could not delete file '{}'", pathname);
        usr::shell::ExitCode::CommandError
    }
}
