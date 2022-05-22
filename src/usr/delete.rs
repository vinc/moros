use crate::usr;
use crate::api::syscall;
use crate::api::fs;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() != 2 {
        return usr::shell::ExitCode::CommandError;
    }

    let mut pathname = args[1];

    // The commands `delete /usr/alice/` and `delete /usr/alice` are equivalent,
    // but `delete /` should not be modified.
    if pathname.len() > 1 {
        pathname = pathname.trim_end_matches('/');
    }

    if !fs::exists(pathname) {
        error!("File not found '{}'", pathname);
        return usr::shell::ExitCode::CommandError;
    }

    if let Some(info) = syscall::info(pathname) {
        if info.is_dir() && info.size() > 0 {
            error!("Directory '{}' not empty", pathname);
            return usr::shell::ExitCode::CommandError;
        }
    }

    if fs::delete(pathname).is_ok() {
        usr::shell::ExitCode::CommandSuccessful
    } else {
        error!("Could not delete file '{}'", pathname);
        usr::shell::ExitCode::CommandError
    }
}
