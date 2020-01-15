use crate::{print, kernel, user};

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() != 2 {
        return user::shell::ExitCode::CommandError;
    }

    let mut pathname = args[1];

    // The commands `list /usr/alice/` and `list /usr/alice` are equivalent,
    // but `list /` should not be modified.
    if pathname.len() > 1 {
        pathname = pathname.trim_end_matches('/');
    }

    if let Some(dir) = kernel::fs::Dir::open(pathname) {
        for file in dir.read() {
            print!("{}\n", file.name());
        }
        user::shell::ExitCode::CommandSuccessful
    } else {
        print!("Dir not found '{}'\n", pathname);
        user::shell::ExitCode::CommandError
    }
}
