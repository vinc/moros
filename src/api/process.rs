use crate::api::syscall;
use crate::api::fs;

pub fn spawn(path: &str) -> Result<(), ()> {
    if let Ok(path) = fs::canonicalize(path) {
        if syscall::stat(&path).is_some() {
            syscall::spawn(&path);
            return Ok(());
        }
    }
    Err(())
}
