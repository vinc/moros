use crate::api::syscall;

pub fn spawn(path: &str) -> Result<(), ()> {
    if syscall::stat(&path).is_some() {
        syscall::spawn(&path);
        return Ok(());
    }
    Err(())
}
