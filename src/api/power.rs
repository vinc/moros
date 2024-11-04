use crate::api::syscall;

pub fn reboot() {
    syscall::stop(0xCAFE);
}

pub fn halt() {
    syscall::stop(0xDEAD);
}
