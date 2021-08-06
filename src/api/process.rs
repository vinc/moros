use crate::sys::process::Process;
use crate::api::fs;

pub fn create(path: &str) -> Result<Process, ()> {
    if let Ok(bin) = fs::read(path) {
        Ok(Process::create(&bin))
    } else {
        Err(())
    }
}
