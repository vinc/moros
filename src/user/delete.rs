use crate::{kernel, print, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let mut pathname = args[1];

    if pathname.starts_with("/dev") || pathname.starts_with("/sys") {
        print!("Permission denied to delete '{}'\n", pathname);
        return user::shell::ExitCode::CommandError;
    }

    // The commands `delete /usr/alice/` and `delete /usr/alice` are equivalent,
    // but `delete /` should not be modified.
    if pathname.len() > 1 {
        pathname = pathname.trim_end_matches('/');
    }

    if let Some(dir) = kernel::fs::Dir::open(pathname) {
        if dir.read().count() == 0 {
            if kernel::fs::Dir::delete(pathname).is_ok() {
                user::shell::ExitCode::CommandSuccessful
            } else {
                print!("Could not delete directory '{}'\n", pathname);
                user::shell::ExitCode::CommandError
            }
        } else {
            print!("Directory '{}' not empty\n", pathname);
            user::shell::ExitCode::CommandError
        }
    } else if kernel::fs::File::open(pathname).is_some() {
        if kernel::fs::File::delete(pathname).is_ok() {
            user::shell::ExitCode::CommandSuccessful
        } else {
            print!("Could not delete file '{}'\n", pathname);
            user::shell::ExitCode::CommandError
        }
    } else {
        print!("File not found '{}'\n", pathname);
        user::shell::ExitCode::CommandError
    }
}
