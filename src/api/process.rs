use crate::api::syscall;

pub fn spawn(path: &str, args: &[&str]) -> Result<usize, usize> {
    if syscall::info(path).is_some() {
        syscall::spawn(path, args)
    } else {
        Err(1)
    }
}
