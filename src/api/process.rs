use crate::api::syscall;

pub const OPEN_ERROR: usize = 128;
pub const READ_ERROR: usize = 129;
pub const EXEC_ERROR: usize = 130;

pub fn spawn(path: &str, args: &[&str]) -> Result<usize, usize> {
    if syscall::info(path).is_some() {
        syscall::spawn(path, args)
    } else {
        Err(OPEN_ERROR)
    }
}
