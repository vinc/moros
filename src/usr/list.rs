use crate::{sys, usr};
use alloc::vec::Vec;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    let current_dir = sys::process::dir();
    let mut pathname = if args.len() == 2 && !args[1].is_empty() {
        args[1]
    } else {
        &current_dir
    };

    // The commands `list /usr/alice/` and `list /usr/alice` are equivalent,
    // but `list /` should not be modified.
    if pathname.len() > 1 {
        pathname = pathname.trim_end_matches('/');
    }

    if let Some(dir) = sys::fs::Dir::open(pathname) {
        let mut files: Vec<_> = dir.read().collect();

        files.sort_by_key(|f| f.name());

        for file in files {
            println!("{}", file.name());
        }
        usr::shell::ExitCode::CommandSuccessful
    } else {
        println!("Dir not found '{}'", pathname);
        usr::shell::ExitCode::CommandError
    }
}
