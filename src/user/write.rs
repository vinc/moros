use crate::{print, kernel, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let pathname = args[1];
    if pathname.starts_with("/dev") || pathname.starts_with("/sys") {
        print!("Permission denied to write to '{}'\n", pathname);
        user::shell::ExitCode::CommandError
    } else if let Some(mut file) = kernel::fs::File::create(pathname) {
        file.write("fake contents");
        user::shell::ExitCode::CommandSuccessful
    } else {
        print!("Could not write to '{}'\n", pathname);
        user::shell::ExitCode::CommandError
    }
}
