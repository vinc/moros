use crate::api::process::ExitCode;
use crate::api::syscall;
use crate::api::fs;

pub fn main(args: &[&str]) -> Result<(), ExitCode> {
    if args.len() < 2 {
        return Err(ExitCode::UsageError);
    }

    for arg in &args[1..] {
        let mut pathname = *arg;

        // The commands `delete /usr/alice/` and `delete /usr/alice` are equivalent,
        // but `delete /` should not be modified.
        if pathname.len() > 1 {
            pathname = pathname.trim_end_matches('/');
        }

        if !fs::exists(pathname) {
            error!("File not found '{}'", pathname);
            return Err(ExitCode::Failure);
        }

        if let Some(info) = syscall::info(pathname) {
            if info.is_dir() && info.size() > 0 {
                error!("Directory '{}' not empty", pathname);
                return Err(ExitCode::Failure);
            }
        }

        if fs::delete(pathname).is_err() {
            error!("Could not delete file '{}'", pathname);
            return Err(ExitCode::Failure);
        }
    }
    Ok(())
}
