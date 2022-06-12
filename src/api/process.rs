use crate::api::syscall;

pub fn spawn(path: &str, args: &[&str]) -> Result<(), ()> {
    if syscall::info(path).is_some() {
        return syscall::spawn(path, args);
    }
    Err(())
}
