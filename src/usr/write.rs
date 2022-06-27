use crate::api::fs;
use crate::api::process;
use crate::api::syscall;

pub fn main(args: &[&str]) -> Result<(), usize> {
    if args.len() != 2 {
        return Err(process::EXIT_FAILURE);
    }

    let pathname = args[1];

    // The command `write /usr/alice/` with a trailing slash will create
    // a directory, while the same command without a trailing slash will
    // create a file.
    let res = if pathname.ends_with('/') {
        let pathname = pathname.trim_end_matches('/');
        fs::create_dir(pathname)
    } else {
        fs::create_file(pathname)
    };

    if let Some(handle) = res {
        syscall::close(handle);
        Ok(())
    } else {
        error!("Could not write to '{}'", pathname);
        Err(process::EXIT_FAILURE)
    }
}
