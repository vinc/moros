use crate::{kernel, print, user};
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    let current_dir = kernel::process::dir();
    let mut pathname = if args.len() == 2 && args[1].len() > 0 {
        args[1]
    } else {
        &current_dir
    };

    // The commands `list /usr/alice/` and `list /usr/alice` are equivalent,
    // but `list /` should not be modified.
    if pathname.len() > 1 {
        pathname = pathname.trim_end_matches('/');
    }

    if let Some(dir) = kernel::fs::Dir::open(pathname) {
        let mut files: Vec<_> = dir.read().collect();

        files.sort_by_key(|f| f.name());

        for file in files {
            print!("{}\n", file.name());
        }
        user::shell::ExitCode::CommandSuccessful
    } else {
        print!("Dir not found '{}'\n", pathname);
        user::shell::ExitCode::CommandError
    }
}
