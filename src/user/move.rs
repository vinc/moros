use crate::{print, kernel, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 3 {
        return user::shell::ExitCode::CommandError;
    }

    let from = args[1];
    let to = args[2];

    if to.starts_with("/dev") || to.starts_with("/sys") {
        print!("Permission denied to write to '{}'\n", to);
        user::shell::ExitCode::CommandError
    } else {
        if let Some(file_from) = kernel::fs::File::open(from) {
            if let Some(mut file_to) = kernel::fs::File::create(to) {
                file_to.write(&file_from.read());
                user::shell::ExitCode::CommandSuccessful
            } else {
                print!("Permission denied to write to '{}'\n", to);
                user::shell::ExitCode::CommandError
            }
        } else {
            print!("File not found '{}'\n", from);
            user::shell::ExitCode::CommandError
        }
    }
}
