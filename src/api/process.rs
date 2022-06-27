use crate::api::syscall;

pub const EXIT_SUCCESS: usize = 0;
pub const EXIT_FAILURE: usize = 1;
pub const EXIT_USAGE_ERROR: usize = 64;
pub const EXIT_DATA_ERROR: usize = 65;
pub const EXIT_OPEN_ERROR: usize = 128;
pub const EXIT_READ_ERROR: usize = 129;
pub const EXIT_EXEC_ERROR: usize = 130;

pub fn spawn(path: &str, args: &[&str]) -> Result<(), usize> {
    if syscall::info(path).is_some() {
        syscall::spawn(path, args)
    } else {
        Err(EXIT_OPEN_ERROR)
    }
}
